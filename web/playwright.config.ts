import { defineConfig } from '@playwright/test';
export default defineConfig({testDir:'./tests',use:{channel:'chrome',baseURL:'http://127.0.0.1:3155'},webServer:{env:{C2_LISTEN:'127.0.0.1:3155'},command:'cargo run --manifest-path ../service/Cargo.toml --offline',url:'http://127.0.0.1:3155',reuseExistingServer:!process.env.CI,timeout:120000}});

