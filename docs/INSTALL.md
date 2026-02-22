# Instalación de KUJAV en tu PC

## Requisitos
- Rust toolchain (cargo + rustc)
- Java 17+

## Instalar desde código fuente

```bash
git clone <tu-repo-de-kujav>
cd kujav-compiler
cargo install --path .
```

Esto instala el binario en `~/.cargo/bin/kujav-compiler`.

Si quieres usarlo como `kujav`, crea un alias o symlink:

```bash
ln -s ~/.cargo/bin/kujav-compiler ~/.cargo/bin/kujav
```

Asegúrate de tener `~/.cargo/bin` en tu `PATH`.

## Verificar instalación

```bash
kujav build
```

o, si no creaste symlink:

```bash
kujav-compiler build
```

## Flujo de uso básico

```bash
kujav new mi_app
cd mi_app
kujav build
kujav run
```
