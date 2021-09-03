{pkgs}: {
    deps = [
        pkgs.rustup
		pkgs.valgrind
        pkgs.binaryen
        pkgs.nodejs
        pkgs.nodePackages.npm
        pkgs.gzip
    ];
}