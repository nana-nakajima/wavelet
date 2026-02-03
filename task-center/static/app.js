// WAVELET 任务中心 - 前端逻辑

let allTasks = [];
let currentTaskId = null;

// 初始化
document.addEventListener('DOMContentLoaded', () => {
    loadTasks();
    loadStats();
});

// 加载任务列表
async function loadTasks() {
    try {
        const response = await fetch('/api/tasks');
        allTasks = await response.json();
        renderKanban();
    } catch (error) {
        console.error('Failed to load tasks:', error);
        // 如果API不可用，使用示例数据
        allTasks = getSampleTasks();
        renderKanban();
    }
}

// 获取统计数据
async function loadStats() {
    const todo = allTasks.filter(t => t.status === 'todo').length;
    const inProgress = allTasks.filter(t => t.status === 'in_progress').length;
    const done = allTasks.filter(t => t.status === 'done').length;
    document.getElementById('taskStats').textContent = 
        `📋 ${todo} 待办 | 🔄 ${inProgress} 进行中 | ✅ ${done} 已完成 | 📊 ${allTasks.length} 总计`;
}

// 渲染看板
function renderKanban() {
    const columns = ['todo', 'in_progress', 'review', 'done'];
    const columnNames = {
        todo: '📋 待办',
        in_progress: '🔄 进行中',
        review: '👀 审核',
        done: '✅ 完成'
    };
    
    const board = document.getElementById('kanbanBoard');
    board.innerHTML = '';
    
    const filterStatus = document.getElementById('filterStatus').value;
    const filterPriority = document.getElementById('filterPriority').value;
    const filterCategory = document.getElementById('filterCategory').value;
    
    columns.forEach(status => {
        const columnTasks = allTasks.filter(task => {
            if (filterStatus !== 'all' && task.status !== filterStatus) return false;
            if (filterPriority !== 'all' && task.priority !== filterPriority) return false;
            if (filterCategory !== 'all' && task.category !== filterCategory) return false;
            return task.status === status;
        });
        
        const column = document.createElement('div');
        column.className = `kanban-column ${status}`;
        column.innerHTML = `
            <h3>${columnNames[status]} (${columnTasks.length})</h3>
            <div class="tasks-container">
                ${columnTasks.map(task => renderTaskCard(task)).join('')}
                ${columnTasks.length === 0 ? '<div class="empty-column">暂无任务</div>' : ''}
            </div>
        `;
        board.appendChild(column);
    });
    
    loadStats();
}

// 渲染任务卡片
function renderTaskCard(task) {
    const priorityLabels = {
        low: '🟢 低',
        medium: '🟡 中',
        high: '🟠 高',
        urgent: '🔴 紧急'
    };
    
    const categoryLabels = {
        effect: '🎛️ 效果器',
        sampler: '🎵 采样器',
        sequencer: '📝 音序器',
        midi: '🎹 MIDI',
        ui: '🖥️ UI',
        docs: '📚 文档'
    };
    
    return `
        <div class="task-card" onclick="showTaskDetail('${task.id}')">
            <div class="task-title">${escapeHtml(task.title)}</div>
            <div class="task-badges">
                <span class="badge priority-${task.priority}">${priorityLabels[task.priority]}</span>
                <span class="badge">${categoryLabels[task.category]}</span>
            </div>
            <div class="task-meta">
                <span>👤 ${task.assignee || '未分配'}</span>
                <span class="task-comments">💬 ${task.comments ? task.comments.length : 0}</span>
            </div>
        </div>
    `;
}

// 过滤任务
function filterTasks() {
    renderKanban();
}

// 打开新建任务模态框
function openModal(taskId = null) {
    const modal = document.getElementById('taskModal');
    const form = document.getElementById('taskForm');
    const title = document.getElementById('modalTitle');
    
    if (taskId) {
        const task = allTasks.find(t => t.id === taskId);
        if (task) {
            title.textContent = '编辑任务';
            document.getElementById('taskId').value = task.id;
            document.getElementById('taskTitle').value = task.title;
            document.getElementById('taskDescription').value = task.description;
            document.getElementById('taskPriority').value = task.priority;
            document.getElementById('taskCategory').value = task.category;
            document.getElementById('taskAssignee').value = task.assignee || '';
        }
    } else {
        title.textContent = '新建任务';
        form.reset();
        document.getElementById('taskId').value = '';
    }
    
    modal.style.display = 'block';
}

// 关闭新建任务模态框
function closeModal() {
    document.getElementById('taskModal').style.display = 'none';
}

// 保存任务
async function saveTask(event) {
    event.preventDefault();
    
    const taskId = document.getElementById('taskId').value;
    const taskData = {
        title: document.getElementById('taskTitle').value,
        description: document.getElementById('taskDescription').value,
        priority: document.getElementById('taskPriority').value,
        category: document.getElementById('taskCategory').value,
        assignee: document.getElementById('taskAssignee').value || 'Nana'
    };
    
    try {
        let response;
        if (taskId) {
            response = await fetch(`/api/tasks/${taskId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(taskData)
            });
        } else {
            response = await fetch('/api/tasks', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(taskData)
            });
        }
        
        if (response.ok) {
            closeModal();
            loadTasks();
        } else {
            alert('保存失败');
        }
    } catch (error) {
        console.error('Save error:', error);
        // 离线模式：直接更新本地数据
        if (taskId) {
            const index = allTasks.findIndex(t => t.id === taskId);
            if (index !== -1) {
                allTasks[index] = { ...allTasks[index], ...taskData, updated_at: new Date() };
            }
        } else {
            const newTask = {
                id: Date.now().toString(),
                ...taskData,
                status: 'todo',
                created_at: new Date(),
                updated_at: new Date(),
                comments: []
            };
            allTasks.push(newTask);
        }
        closeModal();
        renderKanban();
    }
}

// 显示任务详情
function showTaskDetail(taskId) {
    const task = allTasks.find(t => t.id === taskId);
    if (!task) return;
    
    currentTaskId = taskId;
    
    const priorityLabels = {
        low: '🟢 低',
        medium: '🟡 中',
        high: '🟠 高',
        urgent: '🔴 紧急'
    };
    
    const categoryLabels = {
        effect: '🎛️ 效果器',
        sampler: '🎵 采样器',
        sequencer: '📝 音序器',
        midi: '🎹 MIDI',
        ui: '🖥️ UI',
        docs: '📚 文档'
    };
    
    const statusLabels = {
        todo: '📋 待办',
        in_progress: '🔄 进行中',
        review: '👀 审核',
        done: '✅ 完成'
    };
    
    document.getElementById('detailTitle').textContent = task.title;
    document.getElementById('detailPriority').textContent = priorityLabels[task.priority];
    document.getElementById('detailCategory').textContent = categoryLabels[task.category];
    document.getElementById('detailStatus').textContent = statusLabels[task.status];
    document.getElementById('detailAssignee').textContent = '👤 ' + (task.assignee || '未分配');
    document.getElementById('detailDescription').textContent = task.description || '暂无描述';
    document.getElementById('commentTaskId').value = task.id;
    
    document.getElementById('detailCreated').textContent = new Date(task.created_at).toLocaleString('zh-CN');
    document.getElementById('detailUpdated').textContent = new Date(task.updated_at).toLocaleString('zh-CN');
    
    // 渲染评论
    const comments = task.comments || [];
    document.getElementById('commentsList').innerHTML = comments.map(comment => `
        <div class="comment">
            <div class="comment-header">
                <span class="comment-author">${escapeHtml(comment.author)}</span>
                <span class="comment-time">${new Date(comment.created_at).toLocaleString('zh-CN')}</span>
            </div>
            <div class="comment-content">${escapeHtml(comment.content)}</div>
        </div>
    `).join('') || '<div style="color: #666; text-align: center;">暂无评论</div>';
    
    document.getElementById('detailModal').style.display = 'block';
}

// 关闭任务详情
function closeDetailModal() {
    document.getElementById('detailModal').style.display = 'none';
    currentTaskId = null;
}

// 添加评论
async function addComment(event) {
    event.preventDefault();
    
    if (!currentTaskId) return;
    
    const author = document.getElementById('commentAuthor').value;
    const content = document.getElementById('commentContent').value;
    
    if (!author || !content) return;
    
    try {
        const response = await fetch(`/api/tasks/${currentTaskId}/comments`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ author, content })
        });
        
        if (response.ok) {
            document.getElementById('commentForm').reset();
            loadTasks();
            showTaskDetail(currentTaskId);
        }
    } catch (error) {
        console.error('Comment error:', error);
        // 离线模式
        const task = allTasks.find(t => t.id === currentTaskId);
        if (task) {
            if (!task.comments) task.comments = [];
            task.comments.push({
                id: Date.now().toString(),
                author,
                content,
                created_at: new Date()
            });
            document.getElementById('commentForm').reset();
            loadTasks();
            showTaskDetail(currentTaskId);
        }
    }
}

// 移动任务状态
async function moveTaskStatus(status) {
    if (!currentTaskId) return;
    
    try {
        const response = await fetch(`/api/tasks/${currentTaskId}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ status })
        });
        
        if (response.ok) {
            closeDetailModal();
            loadTasks();
        }
    } catch (error) {
        console.error('Move error:', error);
        const task = allTasks.find(t => t.id === currentTaskId);
        if (task) {
            task.status = status;
            closeDetailModal();
            loadTasks();
        }
    }
}

// 删除任务
async function deleteCurrentTask() {
    if (!currentTaskId) return;
    
    if (!confirm('确定要删除这个任务吗？')) return;
    
    try {
        const response = await fetch(`/api/tasks/${currentTaskId}`, {
            method: 'DELETE'
        });
        
        if (response.ok) {
            closeDetailModal();
            loadTasks();
        }
    } catch (error) {
        console.error('Delete error:', error);
        allTasks = allTasks.filter(t => t.id !== currentTaskId);
        closeDetailModal();
        loadTasks();
    }
}

// 编辑当前任务
function editCurrentTask() {
    if (currentTaskId) {
        closeDetailModal();
        setTimeout(() => openModal(currentTaskId), 200);
    }
}

// HTML转义
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// 点击模态框外部关闭
window.onclick = function(event) {
    if (event.target.classList.contains('modal')) {
        event.target.style.display = 'none';
    }
}

// 示例数据（API不可用时）
function getSampleTasks() {
    return [
        {
            id: '1',
            title: '实现Subtracks采样播放',
            description: '实现8个独立采样播放轨，支持独立音高、滤波器、包络控制',
            status: 'todo',
            priority: 'high',
            category: 'sampler',
            assignee: 'Nana',
            created_at: new Date(),
            updated_at: new Date(),
            comments: []
        },
        {
            id: '2',
            title: '添加Decimator效果器',
            description: '实现采样率降低效果，产生复古数字质感',
            status: 'in_progress',
            priority: 'medium',
            category: 'effect',
            assignee: 'Nana',
            created_at: new Date(),
            updated_at: new Date(),
            comments: [
                { id: 'c1', author: 'Nana', content: '基本实现完成，测试中', created_at: new Date() }
            ]
        },
        {
            id: '3',
            title: '更新Tonverk文档',
            description: '更新完整功能对齐文档',
            status: 'done',
            priority: 'low',
            category: 'docs',
            assignee: 'Nana',
            created_at: new Date(Date.now() - 86400000),
            updated_at: new Date(Date.now() - 86400000),
            comments: []
        },
        {
            id: '4',
            title: '测试音频分析模块',
            description: '验证RMS、延迟、频谱分析等测试用例',
            status: 'review',
            priority: 'medium',
            category: 'effect',
            assignee: 'Nana',
            created_at: new Date(Date.now() - 172800000),
            updated_at: new Date(Date.now() - 43200000),
            comments: []
        }
    ];
}
