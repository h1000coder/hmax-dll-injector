const { invoke } = window.__TAURI__.core;

// Função para adicionar logs no console da UI
function addLog(msg) {
    const consoleBox = document.getElementById('console');
    const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    const p = document.createElement('p');
    p.textContent = `[${time}] > ${msg}`;
    consoleBox.appendChild(p);
    consoleBox.scrollTop = consoleBox.scrollHeight;
}

async function refreshProcesses() {
    const select = document.getElementById('process-select');
    try {
        const processes = await invoke('get_process_list');
        select.innerHTML = '<option value="">-- Selecione um Processo --</option>';
        
        processes.forEach(proc => {
            if (proc.name.length > 0) {
                const option = document.createElement('option');
                option.value = proc.pid;
                option.textContent = `[${proc.pid}] ${proc.name}`;
                select.appendChild(option);
            }
        });
        addLog("Lista de processos atualizada.");
    } catch (err) {
        addLog(`Erro ao listar processos: ${err}`);
    }
}

async function selectDLL() {
    const dllPathInput = document.getElementById('dll-path');
    try {
        const path = await invoke('select_dll_file');
        if (path && path !== "") {
            dllPathInput.value = path;
            addLog(`DLL selecionada: ${path.split('\\').pop()}`);
        } else {
            addLog('Seleção de DLL cancelada.');
        }
    } catch (error) {
        addLog(`Erro ao selecionar DLL: ${error}`);
    }
}


// O SEGREDO ESTÁ AQUI: Vincular os eventos corretamente
window.addEventListener("DOMContentLoaded", () => {

    // 1. Botão de Atualizar Processos
    document.getElementById('btn-refresh').addEventListener('click', refreshProcesses);
    
    // 2. Botão de Procurar DLL (O ID correto do seu HTML é select-dll-btn)
    document.getElementById('select-dll-btn').addEventListener('click', selectDLL);

    // 3. Botão de Injetar (Já deixamos pronto)
    document.getElementById('btn-inject').addEventListener('click', async () => {
        const pid = document.getElementById('process-select').value;
        const path = document.getElementById('dll-path').value;
        if(pid && path) {
            addLog("Iniciando injeção...");
            const res = await invoke('inject_dll', { pid: parseInt(pid), path });
            addLog(res);
        } else {
            addLog("Selecione um processo e uma DLL!");
        }
    });


    // 4. Injetar DLL
    document.getElementById('btn-inject').addEventListener('click', async () => {
      const pidInput = document.getElementById('process-select');
      const path = document.getElementById('dll-path').value;


      if (!pidInput || !path) {
          addLog("Por favor, selecione um processo e uma DLL.");
          return;
      }

      const pid = parseInt(pidInput.value)
      addLog(`Injetando DLL no processo ${pid}...`);

      try {
        const result = await invoke('inject_dll', { pid: pid, path: path });
        addLog(result);
      } catch (error) {
        addLog(`Erro ao injetar DLL: ${error}`);
      }
    })
    // Carregar processos ao iniciar
    refreshProcesses();
});