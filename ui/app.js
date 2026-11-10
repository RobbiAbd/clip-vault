const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

let currentClips = [];
let selectedClipId = null;
let searchTimeout = null;

async function loadHistory() {
    try {
        const clips = await invoke('get_history', { limit: 100, offset: 0 });
        currentClips = clips;
        renderClips(clips);
        await updateStats();
    } catch (error) {
        console.error('Failed to load history:', error);
        document.getElementById('clipsList').innerHTML = 
            '<div class="empty-state">Failed to load clipboard history</div>';
    }
}

function renderClips(clips) {
    const container = document.getElementById('clipsList');
    
    if (clips.length === 0) {
        container.innerHTML = '<div class="empty-state">No clipboard history yet. Copy something to get started!</div>';
        return;
    }

    container.innerHTML = clips.map(clip => `
        <div class="clip-item ${clip.pinned ? 'pinned' : ''}" data-id="${clip.id}">
            <div class="clip-header">
                <div class="clip-meta">
                    <span class="clip-type">${formatContentType(clip.content_type)}</span>
                    ${clip.pinned ? '<span class="clip-pin-badge">📌 Pinned</span>' : ''}
                    ${clip.app_source ? `<span>from ${clip.app_source}</span>` : ''}
                </div>
                <div class="clip-actions">
                    <button class="btn btn-icon btn-secondary copy-btn" data-id="${clip.id}" title="Copy">
                        📋
                    </button>
                    <button class="btn btn-icon btn-danger delete-btn" data-id="${clip.id}" title="Delete">
                        🗑️
                    </button>
                </div>
            </div>
            <div class="clip-preview">${escapeHtml(clip.preview)}</div>
            <div class="clip-timestamp">${formatTimestamp(clip.timestamp)}</div>
        </div>
    `).join('');

    container.querySelectorAll('.clip-item').forEach(item => {
        item.addEventListener('click', (e) => {
            if (!e.target.closest('.clip-actions')) {
                showDetail(parseInt(item.dataset.id));
            }
        });
    });

    container.querySelectorAll('.copy-btn').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            await copyClip(parseInt(btn.dataset.id));
        });
    });

    container.querySelectorAll('.delete-btn').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            await deleteClip(parseInt(btn.dataset.id));
        });
    });
}

async function showDetail(id) {
    selectedClipId = id;
    const panel = document.getElementById('detailPanel');
    const content = document.getElementById('detailContent');
    
    try {
        const fullContent = await invoke('get_entry', { id });
        
        if (fullContent) {
            if (fullContent.startsWith('data:image')) {
                content.innerHTML = `<img src="${fullContent}" alt="Clipboard image" />`;
            } else {
                content.textContent = fullContent;
            }
        } else {
            content.textContent = 'No content available';
        }
        
        const clip = currentClips.find(c => c.id === id);
        const pinBtn = document.getElementById('pinDetailBtn');
        if (clip) {
            pinBtn.textContent = clip.pinned ? 'Unpin' : 'Pin';
        }
        
        panel.classList.remove('hidden');
    } catch (error) {
        console.error('Failed to load detail:', error);
        content.textContent = 'Failed to load content';
    }
}

function hideDetail() {
    document.getElementById('detailPanel').classList.add('hidden');
    selectedClipId = null;
}

async function copyClip(id) {
    try {
        const content = await invoke('get_entry', { id });
        if (content && !content.startsWith('data:image')) {
            await invoke('copy_to_clipboard', { content });
            console.log('Copied to clipboard');
        }
    } catch (error) {
        console.error('Failed to copy:', error);
    }
}

async function deleteClip(id) {
    if (confirm('Are you sure you want to delete this clip?')) {
        try {
            await invoke('delete_entry', { id });
            if (selectedClipId === id) {
                hideDetail();
            }
            await loadHistory();
        } catch (error) {
            console.error('Failed to delete:', error);
        }
    }
}

async function togglePin(id) {
    try {
        await invoke('toggle_pin', { id });
        await loadHistory();
        if (selectedClipId === id) {
            showDetail(id);
        }
    } catch (error) {
        console.error('Failed to toggle pin:', error);
    }
}

async function clearHistory() {
    if (confirm('Clear all clipboard history? Pinned items will be kept.')) {
        try {
            await invoke('clear_history', { keepPinned: true });
            hideDetail();
            await loadHistory();
        } catch (error) {
            console.error('Failed to clear history:', error);
        }
    }
}

async function searchClips(query) {
    if (!query.trim()) {
        renderClips(currentClips);
        return;
    }

    try {
        const results = await invoke('search_history', { query, limit: 100 });
        renderClips(results);
    } catch (error) {
        console.error('Search failed:', error);
    }
}

async function updateStats() {
    try {
        const stats = await invoke('get_stats');
        document.getElementById('statsText').textContent = 
            `${stats.total_clips} clips in history`;
    } catch (error) {
        console.error('Failed to get stats:', error);
    }
}

function formatContentType(type) {
    const types = {
        'Text': '📝 Text',
        'RichText': '📄 Rich Text',
        'Image': '🖼️ Image',
        'Files': '📁 Files',
    };
    return types[type] || type;
}

function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

document.addEventListener('DOMContentLoaded', () => {
    loadHistory();

    document.getElementById('searchInput').addEventListener('input', (e) => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            searchClips(e.target.value);
        }, 300);
    });

    document.getElementById('refreshBtn').addEventListener('click', loadHistory);
    document.getElementById('clearBtn').addEventListener('click', clearHistory);
    document.getElementById('closeDetailBtn').addEventListener('click', hideDetail);
    
    document.getElementById('copyDetailBtn').addEventListener('click', () => {
        if (selectedClipId) copyClip(selectedClipId);
    });
    
    document.getElementById('pinDetailBtn').addEventListener('click', () => {
        if (selectedClipId) togglePin(selectedClipId);
    });
    
    document.getElementById('deleteDetailBtn').addEventListener('click', () => {
        if (selectedClipId) deleteClip(selectedClipId);
    });

    listen('clipboard-update', () => {
        console.log('Clipboard updated');
        loadHistory();
    });
});
