{pkgs}: {
    deps = [
        pkgs.rustup
        pkgs.valgrind
        pkgs.binaryen
        pkgs.nodejs
        pkgs.nodePackages.npm
        pkgs.nodePackages.http-server
        pkgs.gzip
    ];
}