import { t, setLanguage, formatRelativeTime } from './i18n.js';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const HISTORY_LIMIT = 50;
const SEARCH_DEBOUNCE_MS = 150;

const search = document.getElementById('popupSearch');
const list = document.getElementById('popupList');
const footer = document.getElementById('popupFooter');

let clips = [];
let selectedIndex = 0;
let searchTimer = null;

function applyTheme(pref) {
    const dark = pref === 'dark' || (pref === 'system' && matchMedia('(prefers-color-scheme: dark)').matches);
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

async function loadSettings() {
    const settings = await invoke('get_settings');
    applyTheme(settings.theme);
    setLanguage(settings.language);
}

async function loadClips() {
    const query = search.value.trim();
    clips = query
        ? await invoke('search_history', { query, limit: HISTORY_LIMIT })
        : await invoke('get_history', { limit: HISTORY_LIMIT, offset: 0 });
    selectedIndex = Math.min(selectedIndex, Math.max(clips.length - 1, 0));
    render();
}

function render() {
    if (clips.length === 0) {
        const key = search.value.trim() ? 'list.no_results' : 'list.empty';
        list.innerHTML = `<div class="popup-empty">${escapeHtml(t(key))}</div>`;
        return;
    }

    list.innerHTML = clips.map((clip, index) => `
        <div class="popup-item ${index === selectedIndex ? 'selected' : ''}" role="option"
             aria-selected="${index === selectedIndex}" data-index="${index}">
            ${clip.thumbnail
                ? `<img class="popup-thumb" src="${clip.thumbnail}" alt="${escapeHtml(t('clip.image_alt'))}" />`
                : `<div class="popup-text">${escapeHtml(clip.preview)}</div>`}
            <div class="popup-meta">${clip.pinned ? '📌 ' : ''}${escapeHtml(formatRelativeTime(clip.timestamp))}</div>
        </div>
    `).join('');
}

function select(index) {
    if (clips.length === 0) return;
    selectedIndex = Math.min(clips.length - 1, Math.max(0, index));
    list.querySelectorAll('.popup-item').forEach(item => {
        const selected = Number(item.dataset.index) === selectedIndex;
        item.classList.toggle('selected', selected);
        item.setAttribute('aria-selected', String(selected));
        if (selected) item.scrollIntoView({ block: 'nearest' });
    });
}

async function paste(index) {
    const clip = clips[index];
    if (!clip) return;
    try {
        await invoke('paste_entry', { id: clip.id });
    } catch (error) {
        console.error('Failed to paste:', error);
        footer.textContent = t('notify.paste_failed', { error });
        footer.classList.add('error');
    }
}

function resetFooter() {
    footer.classList.remove('error');
    footer.textContent = t('stats.hint');
}

async function onShown() {
    search.value = '';
    selectedIndex = 0;
    try {
        await loadSettings();
        await loadClips();
    } catch (error) {
        console.error('Failed to load popup:', error);
    }
    resetFooter();
    list.scrollTop = 0;
    search.focus();
}

list.addEventListener('mousemove', (e) => {
    const item = e.target.closest('.popup-item');
    if (item && Number(item.dataset.index) !== selectedIndex) select(Number(item.dataset.index));
});

list.addEventListener('click', (e) => {
    const item = e.target.closest('.popup-item');
    if (item) paste(Number(item.dataset.index));
});

search.addEventListener('input', () => {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
        selectedIndex = 0;
        loadClips().catch(error => console.error('Search failed:', error));
    }, SEARCH_DEBOUNCE_MS);
});

document.addEventListener('keydown', (e) => {
    switch (e.key) {
        case 'ArrowDown':
        case 'ArrowUp':
            e.preventDefault();
            select(selectedIndex + (e.key === 'ArrowDown' ? 1 : -1));
            break;
        case 'Enter':
            e.preventDefault();
            paste(selectedIndex);
            break;
        case 'Escape':
            e.preventDefault();
            invoke('hide_popup');
            break;
    }
});

document.getElementById('openMainBtn').addEventListener('click', () => invoke('open_main_window'));

listen('popup-shown', onShown);
listen('clipboard-update', () => {
    loadClips().catch(error => console.error('Failed to refresh popup:', error));
});

onShown();
