import { defineConfig } from '@playwright/test';
export default defineConfig({
 testDir:'./tests',
 use:{channel:'chrome',baseURL:'http://127.0.0.1:3155'},
 webServer:[
  {env:{C2_LISTEN:'127.0.0.1:3155'},command:'cargo run --manifest-path ../service/Cargo.toml --offline',url:'http://127.0.0.1:3155',reuseExistingServer:false,timeout:120000},
  {command:'cargo run --manifest-path ../service/Cargo.toml --example registered_policy --offline',url:'http://127.0.0.1:3156',reuseExistingServer:false,timeout:120000}
 ]
});
