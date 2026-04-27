(function () {
  'use strict';

  const STORAGE_KEY = 'ai_chat_conversations';
  const STORAGE_ACTIVE = 'ai_chat_active';

  const $ = (sel) => document.querySelector(sel);
  const $$ = (sel) => document.querySelectorAll(sel);

  // ── DOM Refs ──
  const sidebar    = $('#sidebar');
  const overlay    = $('#overlay');
  const convList   = $('#convList');
  const chatMain   = $('#chatMain');
  const messagesEl = $('#messages');
  const welcome    = $('#welcome');
  const msgInput   = $('#msgInput');
  const btnSend    = $('#btnSend');
  const btnNewChat = $('#btnNewChat');
  const btnToggle  = $('#btnToggleSidebar');
  const btnDelete  = $('#btnDeleteConv');
  const btnClearAll = $('#btnClearAll');
  const searchInput = $('#searchInput');
  const chatHeaderName  = $('#chatHeaderName');
  const chatHeaderStatus = $('#chatHeaderStatus');
  const typingEl = document.createElement('div');
  typingEl.className = 'typing-indicator hidden';
  typingEl.innerHTML = '<div class="typing-avatar">AI</div><div class="typing-dots"><span class="typing-dot"></span><span class="typing-dot"></span><span class="typing-dot"></span></div>';
  messagesEl.after(typingEl);

  // ── State ──
  let conversations = [];
  let activeConvId = null;
  let isTyping = false;

  // ── Bot Knowledge Base ──
  const BOT_RESPONSES = {
    greeting: [
      '你好！很高兴和你交流，有什么我可以帮你的吗？',
      '嗨！今天过得怎么样？有什么问题想问我吗？',
      '你好呀！我是AI助手，随时为你服务～',
    ],
    farewell: [
      '再见！祝你有美好的一天！',
      '下次再聊，拜拜！',
      '好的，有需要随时找我哦～',
    ],
    thanks: [
      '不客气！能帮到你我很开心。',
      '不用谢，这是我应该做的～',
      '很高兴能帮上忙！',
    ],
    ai_question: [
      '人工智能（AI）是计算机科学的一个分支，旨在创建能够执行通常需要人类智能才能完成的任务的系统。这包括学习、推理、问题解决、感知和语言理解等能力。\n\n简单来说，AI就像一个非常聪明的"程序大脑"，它可以：\n- **学习**: 从大量数据中找出规律\n- **推理**: 根据已知信息做出判断\n- **创造**: 生成文本、图像、音乐等\n\n常见的AI应用包括语音助手（如Siri）、推荐系统（如抖音）、自动驾驶等。目前最热门的是大语言模型（LLM），也就是我这种能够进行自然语言对话的AI。',
    ],
    efficiency: [
      '提高工作效率的几个实用方法：\n\n1. **番茄工作法**：专注25分钟，休息5分钟，循环4次后休息15分钟\n2. **优先级矩阵**：用"重要-紧急"四象限来安排任务\n3. **批量处理**：将相似任务集中处理，减少切换成本\n4. **两分钟法则**：如果一件事两分钟内能完成，立刻做\n5. **避免多任务**：研究表明多任务会降低40%的效率\n6. **定期复盘**：每周花15分钟回顾，调整下周计划\n7. **使用工具**：善用待办清单、笔记软件、自动化脚本\n\n关键在于找到适合自己的方法并坚持执行！',
    ],
    book_recommendation: [
      '以下是我推荐的几本好书：\n\n📖 **《人类简史》- 尤瓦尔·赫拉利**\n从认知革命到科学革命，重新理解人类文明史，视角宏大，发人深省。\n\n📖 **《思考，快与慢》- 丹尼尔·卡尼曼**\n诺贝尔经济学奖得主的经典之作，揭示人类思维的两种模式及其偏误。\n\n📖 **《代码整洁之道》- Robert C. Martin**\n程序员必读，教你写出可读性强、可维护的代码。\n\n📖 **《三体》- 刘慈欣**\n中国科幻里程碑，宏大宇宙观与深刻人性思考的结合。\n\n📖 **《非暴力沟通》- 马歇尔·卢森堡**\n改善人际关系和沟通方式的实用指南。\n\n希望有适合你的！',
    ],
    spring_poem: [
      '春风拂柳绿，\n细雨润花红。\n燕子归檐下，\n人间四月同。\n\n这首五言绝句描绘了春天的生机：轻柔的春风吹绿了柳枝，丝丝细雨滋润着红花。归来的燕子在屋檐下呢喃，人间四月处处是这般美好。',
    ],
    weather: [
      '我目前无法获取实时天气数据，建议你使用天气APP或搜索引擎查询当地天气。',
    ],
    joke: [
      '为什么程序员总是分不清万圣节和圣诞节？\n\n因为 Oct 31 == Dec 25！\n\n（八进制的31等于十进制的25 😄）',
      '一个SQL查询走进酒吧，看到两张表，它走过去问："我能Join你们吗？"',
      '程序员最讨厌康熙的哪个儿子？\n\n——胤禩，因为他是八阿哥（bug）！',
    ],
    code_help: [
      '当然可以帮你！请描述一下你遇到了什么问题，或者直接把代码贴出来，我来帮你分析。',
    ],
    fallback: [
      '这是一个有趣的问题！让我想想...\n\n作为AI助手，我会尽力给你提供准确、有用的信息。你能再详细描述一下你的问题吗？这样我能更好地帮到你。',
      '我理解你的问题。虽然我可能无法给出完美的答案，但我会基于已有的知识尽力帮助你。你能补充一些细节吗？',
      '好问题！我的知识库可能有限，但我会尝试回答。如果你有更具体的要求，欢迎告诉我！',
    ],
  };

  function getBotResponse(text) {
    const t = text.trim().toLowerCase();

    if (/^(你好|hi|hello|嗨|哈喽)/.test(t) && t.length < 10) return randomPick(BOT_RESPONSES.greeting);
    if (/^(再见|拜拜|bye|晚安)/.test(t)) return randomPick(BOT_RESPONSES.farewell);
    if (/^(谢谢|感谢|多谢|thanks)/.test(t)) return randomPick(BOT_RESPONSES.thanks);
    if (/人工智|什么是ai|ai是什么|什么是人工智能/.test(t)) return randomPick(BOT_RESPONSES.ai_question);
    if (/效率|高效|提高效率/.test(t)) return randomPick(BOT_RESPONSES.efficiency);
    if (/推荐.*书|好书|看书|阅读/.test(t)) return randomPick(BOT_RESPONSES.book_recommendation);
    if (/春天.*诗|写.*诗|作.*诗|五言|七言/.test(t)) return randomPick(BOT_RESPONSES.spring_poem);
    if (/天气/.test(t)) return randomPick(BOT_RESPONSES.weather);
    if (/笑话|段子|幽默/.test(t)) return randomPick(BOT_RESPONSES.joke);
    if (/代码|编程|bug|报错|怎么写|帮我写/.test(t)) return randomPick(BOT_RESPONSES.code_help);

    return randomPick(BOT_RESPONSES.fallback);
  }

  function randomPick(arr) {
    return arr[Math.floor(Math.random() * arr.length)];
  }

  // ── Persistence ──
  function save() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(conversations));
  }

  function load() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (raw) {
        conversations = JSON.parse(raw);
        // 确保数据结构完整
        conversations.forEach(c => {
          if (!c.messages) c.messages = [];
          if (!c.createdAt) c.createdAt = Date.now();
          if (!c.updatedAt) c.updatedAt = Date.now();
        });
      }
    } catch (_) { conversations = []; }

    activeConvId = localStorage.getItem(STORAGE_ACTIVE);
    if (!conversations.find(c => c.id === activeConvId)) {
      activeConvId = conversations.length > 0 ? conversations[0].id : null;
    }
  }

  // ── Helpers ──
  function uid() {
    return Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
  }

  function formatTime(ts) {
    const d = new Date(ts);
    const now = new Date();
    const pad = n => String(n).padStart(2, '0');
    const time = `${pad(d.getHours())}:${pad(d.getMinutes())}`;

    const isToday = d.toDateString() === now.toDateString();
    if (isToday) return time;

    const yesterday = new Date(now); yesterday.setDate(yesterday.getDate() - 1);
    if (d.toDateString() === yesterday.toDateString()) return `昨天 ${time}`;

    return `${d.getMonth() + 1}/${d.getDate()} ${time}`;
  }

  function getPreview(text, maxLen = 30) {
    return text.length > maxLen ? text.slice(0, maxLen) + '...' : text;
  }

  function getActiveConv() {
    return conversations.find(c => c.id === activeConvId) || null;
  }

  // ── Format Bot Message (Simple Markdown-like) ──
  function formatBotContent(text) {
    let html = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Code blocks ``` ... ```
    html = html.replace(/```(\w*)\n?([\s\S]*?)```/g, (_, lang, code) => {
      return `<pre><code class="language-${lang || ''}">${code.trim()}</code></pre>`;
    });

    // Inline code `...`
    html = html.replace(/`([^`]+)`/g, '<code>$1</code>');

    // Bold **...**
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');

    // Italic *...*
    html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');

    // Line breaks
    html = html.replace(/\n/g, '<br>');

    return html;
  }

  // ── Render ──
  function renderConvList(filter = '') {
    convList.innerHTML = '';

    let list = conversations;
    if (filter) {
      const q = filter.toLowerCase();
      list = list.filter(c => c.title.toLowerCase().includes(q));
    }

    if (list.length === 0) {
      convList.innerHTML = '<div class="empty-state"><p>暂无对话</p></div>';
      return;
    }

    list.forEach(c => {
      const lastMsg = c.messages.length > 0 ? c.messages[c.messages.length - 1] : null;
      const preview = lastMsg ? getPreview(lastMsg.content) : '新对话';

      const div = document.createElement('div');
      div.className = 'conv-item' + (c.id === activeConvId ? ' active' : '');
      div.dataset.id = c.id;
      div.innerHTML = `
        <div class="conv-item-icon">AI</div>
        <div class="conv-item-body">
          <div class="conv-item-title">${escapeHtml(c.title)}</div>
          <div class="conv-item-preview">${escapeHtml(preview)}</div>
        </div>
        <span class="conv-item-time">${formatTime(c.updatedAt)}</span>
        <button class="btn-delete-item" data-del="${c.id}" title="删除">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
            <line x1="2" y1="2" x2="12" y2="12"/><line x1="12" y1="2" x2="2" y2="12"/>
          </svg>
        </button>
      `;
      div.addEventListener('click', (e) => {
        if (e.target.closest('[data-del]')) return;
        switchConversation(c.id);
      });
      div.querySelector('.btn-delete-item').addEventListener('click', (e) => {
        e.stopPropagation();
        deleteConversation(c.id);
      });
      convList.appendChild(div);
    });
  }

  function renderMessages() {
    const conv = getActiveConv();
    if (!conv) {
      messagesEl.innerHTML = '';
      welcome.classList.remove('hidden');
      messagesEl.classList.add('hidden');
      btnDelete.classList.add('hidden');
      chatHeaderName.textContent = 'AI 助手';
      chatHeaderStatus.textContent = '在线';
      chatHeaderStatus.className = 'chat-header-status';
      return;
    }

    welcome.classList.add('hidden');
    messagesEl.classList.remove('hidden');
    btnDelete.classList.remove('hidden');
    chatHeaderName.textContent = conv.title;
    chatHeaderStatus.textContent = '在线';
    chatHeaderStatus.className = 'chat-header-status';

    if (conv.messages.length === 0) {
      messagesEl.innerHTML = '<div class="empty-state"><p>开始一段新的对话吧～</p></div>';
      return;
    }

    let html = '<div class="msg-group">';
    conv.messages.forEach(msg => {
      const roleClass = msg.role === 'user' ? 'user' : 'bot';
      const avatarContent = msg.role === 'user' ? '' : '<div class="msg-avatar">AI</div>';
      const bubbleContent = msg.role === 'user'
        ? escapeHtml(msg.content)
        : formatBotContent(msg.content);

      html += `
        <div class="msg-row ${roleClass}" data-id="${msg.id}">
          ${avatarContent}
          <div>
            <div class="msg-bubble">${bubbleContent}</div>
            <div class="msg-time">${formatTime(msg.time)}</div>
          </div>
        </div>
      `;
    });
    html += '</div>';
    messagesEl.innerHTML = html;
  }

  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }

  function scrollToBottom() {
    requestAnimationFrame(() => {
      messagesEl.scrollTop = messagesEl.scrollHeight;
    });
  }

  // ── Typing Indicator ──
  function showTyping() {
    isTyping = true;
    typingEl.classList.remove('hidden');
    chatHeaderStatus.textContent = '正在输入...';
    chatHeaderStatus.className = 'chat-header-status';
    scrollToBottom();
  }

  function hideTyping() {
    isTyping = false;
    typingEl.classList.add('hidden');
    chatHeaderStatus.textContent = '在线';
    chatHeaderStatus.className = 'chat-header-status';
  }

  // ── Conversation CRUD ──
  function createConversation() {
    const conv = {
      id: uid(),
      title: '新对话',
      messages: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    conversations.unshift(conv);
    activeConvId = conv.id;
    save();
    localStorage.setItem(STORAGE_ACTIVE, activeConvId);
    refreshAll();
    msgInput.focus();
  }

  function switchConversation(id) {
    activeConvId = id;
    localStorage.setItem(STORAGE_ACTIVE, activeConvId);
    refreshAll();
    msgInput.focus();
    closeSidebar();
  }

  function deleteConversation(id) {
    showDialog('确认删除', '删除后对话记录将无法恢复，确定要删除吗？', true, () => {
      conversations = conversations.filter(c => c.id !== id);
      if (activeConvId === id) {
        activeConvId = conversations.length > 0 ? conversations[0].id : null;
        localStorage.setItem(STORAGE_ACTIVE, activeConvId || '');
      }
      save();
      refreshAll();
    });
  }

  function clearAllConversations() {
    if (conversations.length === 0) return;
    showDialog('清空所有对话', '这将删除所有对话记录且无法恢复，确定继续吗？', true, () => {
      conversations = [];
      activeConvId = null;
      localStorage.removeItem(STORAGE_KEY);
      localStorage.removeItem(STORAGE_ACTIVE);
      refreshAll();
    });
  }

  // ── Messages ──
  function addMessage(role, content) {
    let conv = getActiveConv();
    if (!conv) {
      createConversation();
      conv = getActiveConv();
    }

    const msg = {
      id: uid(),
      role,
      content,
      time: Date.now(),
    };
    conv.messages.push(msg);
    conv.updatedAt = Date.now();

    // 自动设置对话标题
    if (conv.messages.length === 1 && role === 'user') {
      conv.title = getPreview(content, 20);
    }

    save();
    renderMessages();
    renderConvList(searchInput.value);
    scrollToBottom();
  }

  async function sendMessage() {
    const text = msgInput.value.trim();
    if (!text || isTyping) return;

    msgInput.value = '';
    autoResize();

    addMessage('user', text);
    await simulateBotResponse(text);
  }

  function simulateBotResponse(userText) {
    return new Promise(resolve => {
      showTyping();

      const delay = Math.min(800 + userText.length * 30, 3000);

      setTimeout(() => {
        const response = getBotResponse(userText);
        hideTyping();
        addMessage('bot', response);
        resolve();
      }, delay);
    });
  }

  // ── Refresh ──
  function refreshAll() {
    renderConvList(searchInput.value);
    renderMessages();
    scrollToBottom();
  }

  // ── Sidebar ──
  function toggleSidebar() {
    sidebar.classList.toggle('open');
    overlay.classList.toggle('show');
  }

  function closeSidebar() {
    sidebar.classList.remove('open');
    overlay.classList.remove('show');
  }

  // ── Dialog ──
  function showDialog(title, message, showDanger, onConfirm) {
    const existing = $('.dialog-overlay');
    if (existing) existing.remove();

    const overlay = document.createElement('div');
    overlay.className = 'dialog-overlay';
    overlay.innerHTML = `
      <div class="dialog">
        <h3>${escapeHtml(title)}</h3>
        <p>${escapeHtml(message)}</p>
        <div class="dialog-actions">
          <button class="btn-cancel">取消</button>
          <button class="btn-danger">确认</button>
        </div>
      </div>
    `;

    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) overlay.remove();
    });
    overlay.querySelector('.btn-cancel').addEventListener('click', () => overlay.remove());
    overlay.querySelector('.btn-danger').addEventListener('click', () => {
      overlay.remove();
      onConfirm();
    });

    document.body.appendChild(overlay);
  }

  // ── Auto-resize Textarea ──
  function autoResize() {
    msgInput.style.height = 'auto';
    msgInput.style.height = Math.min(msgInput.scrollHeight, 150) + 'px';
  }

  // ── Event Listeners ──
  function init() {
    load();
    refreshAll();

    if (!activeConvId && conversations.length === 0) {
      welcome.classList.remove('hidden');
      messagesEl.classList.add('hidden');
    }

    btnSend.addEventListener('click', sendMessage);
    msgInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
      }
    });
    msgInput.addEventListener('input', autoResize);

    btnNewChat.addEventListener('click', createConversation);
    btnToggle.addEventListener('click', toggleSidebar);
    overlay.addEventListener('click', closeSidebar);
    btnDelete.addEventListener('click', () => {
      if (activeConvId) deleteConversation(activeConvId);
    });
    btnClearAll.addEventListener('click', clearAllConversations);

    searchInput.addEventListener('input', () => {
      renderConvList(searchInput.value);
    });

    // Quick prompts
    $$('.quick-prompt').forEach(btn => {
      btn.addEventListener('click', async () => {
        const prompt = btn.dataset.prompt;
        if (!activeConvId) createConversation();
        msgInput.value = '';
        addMessage('user', prompt);
        await simulateBotResponse(prompt);
      });
    });

    // Keyboard shortcut: Ctrl+N new conversation
    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
        e.preventDefault();
        createConversation();
      }
    });
  }

  init();
})();
