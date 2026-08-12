import { createApp } from 'vue'
import './style.css'
import FloatingApp from './components/floating/FloatingApp.vue'

// Floating layout remains a presentation concern; connection state itself is
// owned by useConnections inside ConnectionsPanel.
createApp(FloatingApp).mount('#app')
