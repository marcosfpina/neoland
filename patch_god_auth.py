content = open("flake.nix").read()

content = content.replace(
    'neoland client',
    'NEOLAND_API_KEY="neoland_admin_53352f54e22da11f63edc17380c7bb48aef08811c0872caa" neoland client'
)

open("flake.nix", "w").write(content)
