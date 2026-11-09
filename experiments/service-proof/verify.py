"""Black-box proof against HTTP, independent finite optimum oracle, no production tests."""
import collections
import itertools
import json
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT='http://127.0.0.1:8767'
observations=[]
core_operations=[]
control_observations=[]
def api(path,body=None,method=None,expected=200):
    data=None if body is None else json.dumps(body).encode()
    request=urllib.request.Request(ROOT+path,data=data,method=method or ('POST' if data is not None else 'GET'),headers={'Content-Type':'application/json'})
    start=time.perf_counter()
    try:
        with urllib.request.urlopen(request,timeout=20) as r: status,raw=r.status,r.read()
    except urllib.error.HTTPError as e: status,raw=e.code,e.read()
    observations.append({'path':path,'status':status,'elapsed_ms':(time.perf_counter()-start)*1000,'response_bytes':len(raw)})
    assert status==expected,(path,status,raw)
    return json.loads(raw)
def wait(job,predicate,seconds=20):
    deadline=time.perf_counter()+seconds
    while time.perf_counter()<deadline:
        v=api('/api/jobs/'+job)
        if predicate(v): return v
        time.sleep(.01)
    raise AssertionError(('timeout',job,v))
def create(body):return api('/api/jobs',body)['id']
def command(job,action,body=None,expected=200):return api('/api/jobs/'+job+'/'+action,{} if body is None else body,expected=expected)
def dispose(job):return api('/api/jobs/'+job,method='DELETE')
def prove(job):
    command(job,'prove')
    v=wait(job,lambda v:not v['busy'] and v['state']!='Unproved')
    core_operations.append({'operation':'prove','state':v['state'],'elapsed_ms':v['last_operation_ms']})
    return v
def exhaust(job):
    while True:
        v=api('/api/jobs/'+job)
        if v['state']=='Exhausted':return v
        command(job,'collect',{'size':3})
        v=wait(job,lambda v:not v['busy'])
        assert not v.get('error'),v
        core_operations.append({'operation':'collect','state':v['state'],'elapsed_ms':v['last_operation_ms'],'confirmed':v['confirmed_count']})

def oracle():
    # Independent fixture: SOV singles .40/.40/.50/.50, stock two each,
    # max four whole jump entries, one ordered pair; fixed nonjump BV=8.30.
    singles=['1T','1S','1F','1Lo']
    values=dict(zip(singles,[40,40,50,50]))
    atoms=singles+['+'.join(p) for p in itertools.product(singles,repeat=2)]
    best=0; expected=set()
    for n in range(5):
        for whole in itertools.combinations_with_replacement(atoms,n):
            if sum('+' in x for x in whole)>1:continue
            stock=collections.Counter(p for x in whole for p in x.split('+'))
            if max(stock.values(),default=0)>2:continue
            bv=sum(values[p]*count for p,count in stock.items())
            if bv>best:best=bv;expected=set()
            if bv==best:expected.add(tuple(sorted(whole)))
    assert best==240 and len(expected)==16
    return expected

def main():
    profiles=api('/api/profiles');assert len(profiles)==6
    plans={'bronze':['1Lz','SSp1','ChSq1'],'silver':['1A','CLSp2','ChSp1','ChSq1'],'gold':['1A','CCoSp3','FSSp3','ChSp1','StSq2'],'masters':['1A','CoSp4','FSSp4','ChSp1','StSq4'],'women':['1A','CoSp4','FSSp4','ChSp1','StSq4','ChSq1'],'men':['1A','CoSp4','FSSp4','ChSp1','StSq4','ChSq1']}
    for p in profiles:
        codes=api('/api/elements',{'profile':p})['codes'];assert codes
        a=api('/api/assess',{'profile':p,'elements':plans[p]})
        assert a['compliance'].get('verdict')=='Compliant',a
        assert isinstance(a['bv_hundredths'],int),a
    codes=api('/api/elements',{'profile':'silver','spin_max':{'CSp':1},'spin_off':['LSp']})['codes']
    assert 'CSp1' in codes and 'CSp2' not in codes and 'CCSp2' in codes and 'LSp1' not in codes
    assert api('/api/assess',{'profile':'bronze','elements':['2A']})['compliance']['verdict']=='Noncompliant'
    api('/api/elements',{'profile':'unknown'},expected=400)
    api('/api/elements',{'profile':3},expected=400)
    api('/api/elements',{'jump_max':{'A':0}},expected=400)
    api('/api/elements',[],expected=400)

    fixture={'profile':'bronze','required':[{'id':'spin-a','code':'CCSp1'},{'id':'spin-b','code':'CLSp1'},{'id':'choreo','code':'ChSq1'}],'jump_off':['W','Lz','A'],'mixed':True}
    expected=oracle();job=create(fixture)
    command(job,'collect',expected=409)
    v=prove(job);assert v['maximum_hundredths']==1070,v
    v=exhaust(job);assert len(v['alternatives'])==16
    def identities(view):return {tuple(sorted(c for c in p['codes'] if c not in ['CCSp1','CLSp1','ChSq1'])) for p in view['alternatives']}
    assert identities(v)==expected
    for p in v['alternatives']:
        assert p['bv_hundredths']==1070 and 'spin-a' in p['provenance'] and 'spin-b' in p['provenance'] and 'choreo' in p['provenance']
        a=api('/api/assess',{'profile':'bronze','elements':p['codes'],'mixed':True})
        assert a['bv_hundredths']==1070 and a['compliance']['verdict']=='Compliant',a
    dispose(job);api('/api/jobs/'+job,expected=404)

    paused=create(dict(fixture,test_gate_collection=True));assert prove(paused)['maximum_hundredths']==1070
    command(paused,'collect');wait(paused,lambda v:v.get('gate_waiting') and v['busy'])
    api('/api/profiles') # Control plane still responsive while a core call is held.
    other=create(fixture);assert prove(other)['maximum_hundredths']==1070
    assert api('/api/jobs/'+paused)['gate_waiting']
    api('/api/jobs',fixture,expected=400) # bounded proof capacity
    start=time.perf_counter();command(paused,'pause');v=wait(paused,lambda v:v['state']=='Paused' and not v['busy']);control_observations.append({'operation':'pause_gate_to_ack','wall_ms':(time.perf_counter()-start)*1000})
    assert v['confirmed_count']==0
    v=exhaust(paused);assert identities(v)==expected and len(v['alternatives'])==16
    dispose(paused);dispose(other)

    cancelled=create(dict(fixture,test_gate_collection=True));prove(cancelled)
    command(cancelled,'collect');wait(cancelled,lambda v:v.get('gate_waiting'))
    start=time.perf_counter();command(cancelled,'cancel');v=wait(cancelled,lambda v:v['state']=='Cancelled' and not v['busy']);control_observations.append({'operation':'cancel_gate_to_ack','wall_ms':(time.perf_counter()-start)*1000})
    assert v['maximum_hundredths']==1070 and not v['alternatives']
    command(cancelled,'collect',expected=409);dispose(cancelled)

    proof_cancel=create({'profile':'men'});command(proof_cancel,'prove');command(proof_cancel,'cancel')
    v=wait(proof_cancel,lambda v:v['state']=='Cancelled' and not v['busy'])
    assert not v['alternatives'] and v['maximum_hundredths'] in (None,13310)
    dispose(proof_cancel)

    dying=create(dict(fixture,test_gate_collection=True));prove(dying);command(dying,'collect');wait(dying,lambda v:v.get('gate_waiting'))
    dispose(dying);api('/api/jobs/'+dying,expected=404)
    expired=create(fixture);time.sleep(4.7)
    v=api('/api/jobs/'+expired);assert v['state']=='Expired',v
    command(expired,'prove',expected=409);dispose(expired)
    Path('observations.json').write_text(json.dumps({'passed':True,'oracle_maximum_hundredths':1070,'oracle_compositions':16,'http_observations':observations,'core_operations':core_operations,'control_observations':control_observations},indent=2))
    print(f'PASS: six-profile selection/manual assessment; exact 16-optimum set; in-call pause/resume/cancel; proof cancellation; isolation; disposal; idle expiry; {len(observations)} HTTP observations')

if __name__=='__main__':main()
