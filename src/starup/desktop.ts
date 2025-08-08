import { isDesktopApp } from '../api/core.ts'

if (isDesktopApp) {
  // find div with data-tauri-decorum-tb and remove it
  setTimeout(() => {
    const decorumDiv = document.querySelector('div[data-tauri-decorum-tb]')
    if (decorumDiv) {
      decorumDiv.remove()
    }
  }, 1000)
}
