// src/components/Layout/Titlebar.tsx
import React from 'react'
import { useDesktopEnv } from './usePlatform.ts'

async function withWindow<T>(fn: (w: any) => Promise<T> | T) {
    if (!(window as any).__TAURI_INTERNALS__) return
    const { getCurrent } = await import('@tauri-apps/plugin-window')
    const win = getCurrent()
    return fn(win)
}

export function Titlebar({ hidden }: { hidden?: boolean }) {
    const { platform } = useDesktopEnv()
    if (hidden) return null

    const height = platform === 'mac' ? 38 : 32
    const onClose = () => void withWindow(w => w.close())
    const onMin   = () => void withWindow(w => w.minimize())
    const onMax   = () => void withWindow(w => w.toggleMaximize())

    return (
        <div
            data-tauri-drag-region
            style={{
                position: 'fixed',
                top: 0, left: 0, right: 0,
                height,
                display: 'flex',
                alignItems: 'center',
                justifyContent: platform === 'mac' ? 'flex-start' : 'flex-end',
                zIndex: 20,
            }}
        >
            {platform === 'mac' ? (
                <div style={{ display: 'flex', gap: 6, padding: 6 }}>
                    <div className="no-drag" style={dot('#ff5f57')} title="Close" onClick={onClose} />
                    <div className="no-drag" style={dot('#febc2e')} title="Minimize" onClick={onMin} />
                    <div className="no-drag" style={dot('#28c840')} title="Zoom" onClick={onMax} />
                </div>
            ) : (
                <div style={{ display: 'flex', alignItems: 'center', gap: 2, paddingRight: 6 }}>
                    <button className="no-drag" style={btn} title="Minimize" onClick={onMin}>—</button>
                    <button className="no-drag" style={btn} title="Maximize" onClick={onMax}>▢</button>
                    <button className="no-drag" style={btn} title="Close" onClick={onClose}>✕</button>
                </div>
            )}
        </div>
    )
}

const dot = (bg: string): React.CSSProperties => ({
    width: 12, height: 12, borderRadius: 6, margin: 6,
    background: bg, cursor: 'pointer',
})

const btn: React.CSSProperties = {
    height: 28, minWidth: 36,
    border: 'none',
    background: 'transparent',
    cursor: 'pointer',
}
