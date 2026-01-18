#!/usr/bin/env python3
import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Pattern to find create_labeled_slider calls with 5 params (missing tooltip)
# Look for: create_labeled_slider(\n"Label",\n min, max, default,\n closure
pattern = r'(create_labeled_slider\(\s*"([^"]+)",\s*[\d.]+,\s*[\d.]+,\s*[\d.]+,\s*)(clone!)'

tooltips = {
    "Repetition Penalty": "Penaliza repetições. Valores altos evitam redundância",
    "Typical P": "Typical sampling. Filtra tokens improváveis",
    "Epsilon Cutoff": "Remove tokens com probabilidade muito baixa",
    "ETA Cutoff": "Variante do epsilon cutoff",
    "Tail Free Sampling": "Remove cauda de baixa probabilidade",
    "Top A": "Top-a sampling. Filtragem adaptativa",
    "Similaridade Mínima": "Score mínimo para usar documentos do RAG",
}

def replacer(match):
    prefix = match.group(1)
    label = match.group(2)
    closure = match.group(3)

    tooltip = tooltips.get(label, "Controla comportamento da geração")
    return f'{prefix}Some("{tooltip}"),\n        {closure}'

result = re.sub(pattern, replacer, content)

with open('src/main.rs', 'w') as f:
    f.write(result)

print("Fixed all slider calls!")
