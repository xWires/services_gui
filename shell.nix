{ pkgs ? import <nixpkgs> {} }:
let
  qtEnv = pkgs.qt6.env "qt-custom-${pkgs.qt6.qtbase.version}" (with pkgs.qt6; [
    qtdeclarative
  ]);
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    libGLX
    qtEnv
  ];

  nativeBuildInputs = with pkgs; [
    qtcreator
  ];

  shellHook = ''
    export QMAKE=${qtEnv}/bin/qmake
    export QML_IMPORT_PATH="${qtEnv}/${pkgs.qt6.qtbase.qtQmlPrefix}"
    export CPATH=${qtEnv}/include:${qtEnv}/include/QtQml:${qtEnv}/include/QtQml/${pkgs.qt6.qtbase.version}:${qtEnv}/include/QtQml/${pkgs.qt6.qtbase.version}/QtQml
  '';
}
