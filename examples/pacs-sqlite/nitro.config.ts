import { defineNitroConfig } from "nitro/config";

export default defineNitroConfig({
    serverDir: './server',
    compatibilityDate: '2024-12-16',
    experimental: {
        database: true
    },
    database: {
        default: {
            connector: 'sqlite',
            options: { name: 'pacs.db' }
        }
    }
})