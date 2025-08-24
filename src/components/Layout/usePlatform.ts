import { useEffect, useState } from 'react'
import { isDesktopApp } from '../../api/core' // adjust path if different

export type Platform = 'mac' | 'win' | 'linux' | 'web'

export function useDesktopEnv() {
    const [platform, setPlatform] = useState<Platform>('web')
    const [decorated, setDecorated] = useState<boolean>(true) // assume native bar until proven otherwise
    const [ready, setReady] = useState(false)

    useEffect(() => {
        void (async () => {
            try {
                // platform
                if (isDesktopApp) {
                    try {
                        const { platform } = await import('@tauri-apps/plugin-os')
                        const p = (await platform()).toLowerCase()
                        setPlatform(p.startsWith('darwin') ? 'mac' : p.startsWith('win') ? 'win' : 'linux')
                    } catch {
                        setPlatform('web')
                    }

                    // decoration
                    try {
                        const { getCurrent } = await import('@tauri-apps/plugin-window')
                        const win = getCurrent()
                        setDecorated(await win.isDecorated())
                    } catch {
                        setDecorated(true)
                    }
                } else {
                    // web fallback
                    const nav = navigator as any
                    const hinted: string | undefined = nav.userAgentData?.platform
                    const base = (hinted || navigator.platform || navigator.userAgent || '').toLowerCase()
                    setPlatform(
                        base.includes('win') ? 'win'
                            : base.includes('mac') ? 'mac'
                                : base.includes('linux') || base.includes('x11') ? 'linux'
                                    : 'web'
                    )
                    setDecorated(true)
                }
            } finally {
                setReady(true)
            }
        })()
    }, [])

    return { isDesktop: isDesktopApp, platform, decorated, ready }
}
