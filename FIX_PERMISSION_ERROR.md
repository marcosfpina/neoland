# 🔴 ERRO CRÍTICO: Permission Denied

## Problema Identificado

```
I/O error Permission denied (os error 13)
```

O servidor **não consegue acessar** o diretório de cache do HuggingFace.

## Solução Aplicada

```bash
mkdir -p ~/.cache/huggingface/hub
chmod 755 ~/.cache/huggingface
```

## Próximos Passos

1. **Reiniciar servidor** (permissões agora corretas)
2. **Enviar nova mensagem** no cliente
3. **Aguardar download** do modelo (~1.1 GB, 2-5 min)

## Como Reiniciar

```bash
# Opção 1: Script automático
./restart-server.sh

# Opção 2: Manual
kill $(ps aux | grep llamachat-server | grep -v grep | awk '{print $2}')
./run-server.sh
```

---

**Status**: Permissões corrigidas. Reinicie o servidor para aplicar.
