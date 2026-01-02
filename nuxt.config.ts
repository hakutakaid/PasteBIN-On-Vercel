export default defineNuxtConfig({
    devtools: {
        enabled: true
    },

    css: [
        '~/assets/css/main.css'
    ],

    modules: ['@nuxtjs/tailwindcss'],

    runtimeConfig: {
        adminPin: process.env.ADMIN_PIN || '0000',
        upstashRedisRestUrl: process.env.UPSTASH_REDIS_REST_URL,
        upstashRedisRestToken: process.env.UPSTASH_REDIS_REST_TOKEN,
        public: {}
    },
    app: {
        head: {
            title: 'Paste Vercel',
            meta: [{
                    name: 'description',
                    content: 'Advanced Link Decryption & Hosting System.'
                },
                {
                    name: 'viewport',
                    content: 'width=device-width, initial-scale=1'
                }
            ],
            link: [{
                rel: 'stylesheet',
                href: 'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&family=Orbitron:wght@400;600;800&family=Inter:wght@400;600&display=swap'
            }]
        }
    }
})