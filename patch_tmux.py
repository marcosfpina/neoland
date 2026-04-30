content = open("flake.nix").read()

content = content.replace(
    'tmux select-pane -t 0',
    'tmux select-pane -t neoland-stack:0.0'
)

open("flake.nix", "w").write(content)
