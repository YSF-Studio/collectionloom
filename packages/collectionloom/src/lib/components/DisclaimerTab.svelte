<script>
import { invoke } from "../api/tauri.js";
import { getResolvedLocale, subscribeLocale } from "../stores/locale.js";
let info = $state({ features: [], appName: "CollectionLoom" });
let loaded = $state(false);
let locale = $state(getResolvedLocale());

async function load() {
    if (loaded) return;
    try {
        info = await invoke("about_info");
    } catch(e) { /* use defaults */ }
    loaded = true;
}
$effect(() => { load(); });
$effect(() => subscribeLocale((_, resolved) => { locale = resolved; }));
</script>

<div class="about" style="max-width:580px;margin:0 auto">
    <div style="text-align:center;margin-bottom:24px">
        <img src="/icon.png" style="width:88px;height:88px;border-radius:18px;margin-bottom:12px;box-shadow:0 4px 20px rgba(0,0,0,0.25)" alt="CollectionLoom" />
        <h3 style="margin:0 0 4px;font-size:20px">
            {info.appName}
            <span style="color:var(--text-muted);font-size:12px;margin-left:8px">v{info.version}</span>
        </h3>
        <p style="color:var(--text-secondary);font-size:13px;margin:0">{locale === 'id' ? 'Perangkat Akuisisi Forensik Portable' : 'Portable Forensic Acquisition Toolkit'}</p>
        <p style="color:var(--text-muted);font-size:11px;margin:6px 0 0">{locale === 'id' ? 'Pengumpulan bukti selaras ISO 27037 untuk macOS, Windows, dan Linux' : 'ISO 27037-aligned evidence collection for macOS, Windows, and Linux'}</p>
    </div>

    <div class="card" style="margin-bottom:12px">
        <h4>{locale === 'id' ? 'Fitur' : 'Features'}</h4>
        <ul style="margin:0;padding-left:20px">
            {#each info.features as f}
                <li style="font-size:13px;color:var(--text-secondary);margin-bottom:6px;line-height:1.4">{f}</li>
            {/each}
        </ul>
    </div>

    <div class="card" style="margin-bottom:12px;border-left:3px solid var(--success)">
        <h4>{locale === 'id' ? 'Privasi' : 'Privacy'}</h4>
        <p style="font-size:13px;color:var(--text-secondary);margin:0;line-height:1.5">{info.privacy}</p>
        <span class="offline-badge" style="margin-top:8px">{locale === 'id' ? 'Sepenuhnya Offline' : 'Fully Offline'}</span>
    </div>

    <div class="card" style="margin-bottom:12px;border-left:3px solid var(--warn)">
        <h4>{locale === 'id' ? 'Pernyataan' : 'Disclaimer'}</h4>
        <p style="font-size:13px;color:var(--text-secondary);margin:0;font-style:italic;line-height:1.5">{info.disclaimer}</p>
    </div>

    <div class="card" style="margin-bottom:12px">
        <h4>{locale === 'id' ? 'Pengembang' : 'Developer'}</h4>
        <p style="font-size:13px;color:var(--text);margin:0 0 4px;font-weight:600">{info.developer}</p>
        <p style="font-size:12px;color:var(--primary);margin:0">{info.build}</p>
    </div>

    <p style="text-align:center;font-size:11px;color:var(--text-muted);margin-top:12px">
        YSF Studio © {new Date().getFullYear()} — {locale === 'id' ? 'Seluruh hak dilindungi' : 'All rights reserved.'}
    </p>
</div>
