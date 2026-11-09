"""Additional public HTTP boundary observations, not production acceptance."""
import json
from pathlib import Path
from verify import api,create,prove,dispose,exhaust,oracle

out={}
job=create({'profile':'masters','jump_off':['A']})
out['infeasible']=prove(job)
assert out['infeasible']['state']=='Infeasible'
assert out['infeasible']['maximum_hundredths'] is None
dispose(job)
job=create({'profile':'bronze','required':[{'id':'keep-lutz','code':'1Lz'}],'exclude':['1Lz']})
out['conflicting_request']=prove(job)
assert out['conflicting_request']['state']=='Failed' and out['conflicting_request']['error']
assert out['conflicting_request']['maximum_hundredths'] is None
dispose(job)
fixture={'profile':'bronze','required':['CCSp1','CLSp1','ChSq1'],'jump_off':['W','Lz','A'],'mixed':True,'jump_max':{'T':1,'S':1,'F':1,'Lo':1},'spin_max':{'CCSp':1,'CLSp':1},'step_max':0,'exclude':['1F+1F']}
job=create(fixture);v=prove(job);assert v['maximum_hundredths']==1070
v=exhaust(job)
actual={tuple(sorted(c for c in p['codes'] if c not in ['CCSp1','CLSp1','ChSq1'])) for p in v['alternatives']}
expected={whole for whole in oracle() if '1F+1F' not in whole}
assert actual==expected and len(actual)==14
out['combined_restrictions']={'maximum_hundredths':v['maximum_hundredths'],'identities':sorted(actual),'count':len(actual)}
dispose(job)
a=api('/api/assess',{'profile':'silver','elements':['CSSp2']})
b=api('/api/assess',{'profile':'silver','elements':['CSSp2'],'spin_max':{'CSSp':0}})
assert a['compliance']['verdict']==b['compliance']['verdict']=='Compliant'
assert a['bv_hundredths']==b['bv_hundredths'] and b['personal_conflicts']!='[]'
out['preference_independent_of_rules']={'without_limit':a,'with_limit':b}
Path('evidence/edge-observations.json').write_text(json.dumps(out,indent=2))
print('PASS: infeasible vs invalid request; 14 independently expected optima with combined limits/exact exclusion; personal conflict separate from compliance and BV')
