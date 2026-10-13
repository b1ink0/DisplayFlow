import { createApp } from 'vue'
import App from './App.vue'
import './styles.css'
import ui from '@nuxt/ui/vue-plugin'
import { initializeAppearance } from '@/composables/useAppearance'
initializeAppearance()
createApp(App).use(ui).mount('#app')
