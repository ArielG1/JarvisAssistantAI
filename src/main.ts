import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import "./assets/styles/main.css"
import "./assets/styles/hud.css"
import "./assets/styles/errors.css"

const app = createApp(App)
app.use(createPinia())
app.mount("#app")
