import { defineConfig } from '@playwright/test';
const port=Number(process.env.C2_TEST_PORT??3157);
const fixturePort=Number(process.env.C2_FIXTURE_PORT??3158);
export default defineConfig({
 testDir:'./tests',
 use:{channel:'chrome',baseURL:`http://127.0.0.1:${port}`},
 webServer:[
  {env:{C2_LISTEN:`127.0.0.1:${port}`},command:'cargo run --manifest-path ../service/Cargo.toml --offline',url:`http://127.0.0.1:${port}`,reuseExistingServer:false,timeout:120000},
  {env:{C2_FIXTURE_LISTEN:`127.0.0.1:${fixturePort}`},command:'cargo run --manifest-path ../service/Cargo.toml --example registered_policy --offline',url:`http://127.0.0.1:${fixturePort}`,reuseExistingServer:false,timeout:120000}
 ]
});
