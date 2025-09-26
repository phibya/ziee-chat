import { isMacOS, isTauriView } from '../api/core.ts'

if (isTauriView) {
  // Observe for div with data-tauri-decorum-tb and handle it when it appears
  const observer = new MutationObserver(() => {
    const decorumDiv = document.querySelector(
      'div[data-tauri-decorum-tb]',
    ) as HTMLDivElement | null
    if (decorumDiv) {
      if (isMacOS) {
        decorumDiv.remove()
      } else {
        decorumDiv.style.width = '100px'
        decorumDiv.style.top = '8px'
        decorumDiv.style.right = '0px'
        decorumDiv.style.left = ''
        decorumDiv.style.zIndex = '10000'
        decorumDiv.style.backdropFilter = 'blur(8px)'
        const dragRegion = decorumDiv.querySelector(
          'div[data-tauri-drag-region]',
        )
        if (dragRegion) {
          dragRegion.remove()
        }
      }
      observer.disconnect()
    }
  })

  observer.observe(document.body, {
    childList: true,
    subtree: true,
  })
}
