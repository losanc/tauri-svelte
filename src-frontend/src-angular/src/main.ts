import { bootstrapApplication } from '@angular/platform-browser'
import { appConfig } from './app/app.config'
import { App } from './app/app'
import { Popout } from './app/popout'

const isPopout = window.location.pathname.startsWith('/popout')
const rootComponent = isPopout ? Popout : App

bootstrapApplication(rootComponent, appConfig).catch((err) => console.error(err))
