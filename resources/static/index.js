// 使用现代ES模块语法并移除jQuery依赖
document.addEventListener('DOMContentLoaded', () => {
    const $word = document.getElementById('word');
    $word?.focus();
});

async function loadAndApplyCSS(url) {
    try {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);

        const sheet = new CSSStyleSheet();
        await sheet.replace(await response.text());

        document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
    } catch (error) {
        console.error('CSS加载失败:', error);
    }
}

// 初始化样式加载
loadAndApplyCSS('./index.css').catch(console.error);

// 渲染性能优化
const shadowStyleCache = new WeakMap();
function createShadowDOM(content) {
    const template = document.createElement('template');
    template.innerHTML = `
      <style>
        :host { 

        }
      </style>
      <div class="dict-content">${content.replace(/\u0000/g, '')}</div>
    `;
    return template.content.cloneNode(true);
}

function render(responseData) {
    const container = document.getElementById('mdx-resp');
    if (!responseData?.data?.length) {
        container.style.display = 'none';
        return;
    }

    const fragment = document.createDocumentFragment();
    responseData.data.forEach(item => {
        const card = document.createElement('div');
        card.className = 'dict-card';

        const header = document.createElement('div');
        header.className = 'dict-header';
        header.textContent = item.dict;

        const shadowHost = document.createElement('div');
        const shadowRoot = shadowHost.attachShadow({ mode: 'open' });
        shadowRoot.adoptedStyleSheets = document.adoptedStyleSheets;
        shadowRoot.appendChild(createShadowDOM(item.content));

        card.append(header, shadowHost);
        fragment.appendChild(card);
    });

    container.replaceChildren(fragment);
    container.style.display = 'block';
}

// 异步请求封装
async function queryMdx(word) {
    const container = document.getElementById('mdx-resp');
    try {
        container.innerHTML = '查询中...';

        const formData = new URLSearchParams();
        formData.append('word', word);

        const response = await fetch('./query', {
            method: 'POST',
            body: formData
        });

        if (!response.ok) throw new Error(`请求失败: ${response.status}`);
        render(await response.json());
    } catch (error) {
        console.error('查询错误:', error);
        container.textContent = '查询失败';
    }
}

// 输入验证
const invalidChars = new Set(['.', '#', '?', '/']);
const validInput = word => word && !invalidChars.has(word);

// 事件处理优化
const debounce = (fn, delay = 300) => {
    let timer;
    return (...args) => {
        clearTimeout(timer);
        timer = setTimeout(() => fn(...args), delay);
    };
};

const handleQuery = debounce(() => {
    const word = document.getElementById('word').value.trim();
    if (validInput(word)) queryMdx(word);
});

// 事件监听
document.addEventListener('keydown', e => {
    if (e.key === 'Enter') handleQuery();
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
        e.preventDefault();
        const $word = document.getElementById('word');
        $word.value = '';
        $word.focus();
        window.scrollTo(0, 0);
    }
});

// 统一URL处理函数
function processCustomLink(href) {
    try {
        // 分步解码处理（应对多次编码的情况）
        let decoded = href;
        while (/%[0-9A-Fa-f]{2}/.test(decoded)) {
            decoded = decodeURIComponent(decoded);
        }
        return decoded;
    } catch (e) {
        console.warn('URL解码失败，使用原始值:', href);
        return href;
    }
}

document.addEventListener('click', e => {
    const link = e.composedPath().find(n => n.tagName === 'A');
    if (!link || !link.href) return;

    const rawHref = link.getAttribute('href'); // 获取未经浏览器处理的原始值
    if (!rawHref) return;

    const decodedHref = processCustomLink(rawHref);

    // 处理entry://协议
    if (decodedHref.startsWith('entry://')) {
        e.preventDefault();
        const word = decodedHref.replace(/^entry:\/\//, '');
        document.getElementById('word').value = word;
        handleQuery();
        return;
    }

});

// lucky btn
document.getElementById('lucky-btn')?.addEventListener('click', async () => {
    try {
        const response = await fetch('./lucky', { method: 'GET' });
        render(await response.json());
    } catch (error) {
        console.error('获取lucky word失败:', error);
    }
});