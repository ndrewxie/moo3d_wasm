{pkgs}: {
    deps = [
        pkgs.rustup
        pkgs.nodejs-16_x
        pkgs.nodePackages.npm
        pkgs.python39
    ];
}