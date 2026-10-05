# Lazyfetch

[![GitHub license](https://img.shields.io/github/license/maslina524/lazyfetch?style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48c3ZnIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIG9wYWNpdHk9IjAuMSIgZD0iTTEyIDE3SDdDNS44OTU0MyAxNyA1IDE2LjEwNDYgNSAxNVY1QzUgMy44OTU0MyA1Ljg5NTQzIDMgNyAzSDE2QzE3LjEwNDYgMyAxOCAzLjg5NTQzIDE4IDVWMTlDMTggMjAuMTA0NiAxNy4xMDQ2IDIxIDE2IDIxQzE0Ljg5NTQgMjEgMTQgMjAuMTA0NiAxNCAxOUMxNCAxNy44OTU0IDEzLjEwNDYgMTcgMTIgMTdaIiBmaWxsPSIjZmZmZmZmIi8+PHBhdGggZD0iTTE5IDNIOVYzQzcuMTE0MzggMyA2LjE3MTU3IDMgNS41ODU3OSAzLjU4NTc5QzUgNC4xNzE1NyA1IDUuMTE0MzggNSA3VjEwLjVWMTciIHN0cm9rZT0iI2ZmZmZmZiIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz48cGF0aCBkPSJNMTQgMTdWMTlDMTQgMjAuMTA0NiAxNC44OTU0IDIxIDE2IDIxVjIxQzE3LjEwNDYgMjEgMTggMjAuMTA0NiAxOCAxOVY5VjQuNUMxOCAzLjY3MTU3IDE4LjY3MTYgMyAxOS41IDNWM0MyMC4zMjg0IDMgMjEgMy42NzE1NyAyMSA0LjVWNC41QzIxIDUuMzI4NDMgMjAuMzI4NCA2IDE5LjUgNkgxOC41IiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+PHBhdGggZD0iTTE2IDIxSDVDMy44OTU0MyAyMSAzIDIwLjEwNDYgMyAxOVYxOUMzIDE3Ljg5NTQgMy44OTU0MyAxNyA1IDE3SDE0IiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+PHBhdGggZD0iTTkgN0gxNCIgc3Ryb2tlPSIjZmZmZmZmIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIvPjxwYXRoIGQ9Ik05IDExSDE0IiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+PC9zdmc+)](https://github.com/maslina524/lazyfetch/blob/main/LICENSE)
[![GitHub top language](https://img.shields.io/github/languages/top/maslina524/lazyfetch?style=for-the-badge&logo=rust&label=Rust)](https://github.com/maslina524/lazyfetch/blob/main/Cargo.toml)
[![GitHub commit activity](https://img.shields.io/github/commit-activity/m/maslina524/lazyfetch?style=for-the-badge&logo=github)](https://github.com/maslina524/lazyfetch/commits)  
[![No std](https://img.shields.io/badge/Built_with-%23!%5Bno__std%5D-blue?style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48IS0tIFVwbG9hZGVkIHRvOiBTVkcgUmVwbywgd3d3LnN2Z3JlcG8uY29tLCBHZW5lcmF0b3I6IFNWRyBSZXBvIE1peGVyIFRvb2xzIC0tPgo8c3ZnIGZpbGw9IiNmZmZmZmYiIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDI0IDI0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik0xNC4yNSw4SDkuNzVBMS43NTIsMS43NTIsMCwwLDAsOCw5Ljc1djQuNUExLjc1MiwxLjc1MiwwLDAsMCw5Ljc1LDE2aDQuNUExLjc1MiwxLjc1MiwwLDAsMCwxNiwxNC4yNVY5Ljc1QTEuNzUyLDEuNzUyLDAsMCwwLDE0LjI1LDhaTTE0LDE0SDEwVjEwaDRabTgtNWExLDEsMCwwLDAsMC0ySDIwVjYuNzVBMi43NTIsMi43NTIsMCwwLDAsMTcuMjUsNEgxN1YyYTEsMSwwLDAsMC0yLDBWNEgxM1YyYTEsMSwwLDAsMC0yLDBWNEg5VjJBMSwxLDAsMCwwLDcsMlY0SDYuNzVBMi43NTIsMi43NTIsMCwwLDAsNCw2Ljc1VjdIMkExLDEsMCwwLDAsMiw5SDR2MkgyYTEsMSwwLDAsMCwwLDJINHYySDJhMSwxLDAsMCwwLDAsMkg0di4yNUEyLjc1MiwyLjc1MiwwLDAsMCw2Ljc1LDIwSDd2MmExLDEsMCwwLDAsMiwwVjIwaDJ2MmExLDEsMCwwLDAsMiwwVjIwaDJ2MmExLDEsMCwwLDAsMiwwVjIwaC4yNUEyLjc1MiwyLjc1MiwwLDAsMCwyMCwxNy4yNVYxN2gyYTEsMSwwLDAsMCwwLTJIMjBWMTNoMmExLDEsMCwwLDAsMC0ySDIwVjlabS00LDguMjVhLjc1MS43NTEsMCwwLDEtLjc1Ljc1SDYuNzVBLjc1MS43NTEsMCwwLDEsNiwxNy4yNVY2Ljc1QS43NTEuNzUxLDAsMCwxLDYuNzUsNmgxMC41YS43NTEuNzUxLDAsMCwxLC43NS43NVoiLz48L3N2Zz4=)](https://github.com/maslina524/lazyfetch/blob/main/src/main.rs)
[![Clippy](https://img.shields.io/badge/Clippy-%23!%5Bdeny%28clippy::all%29%5D-blue?style=for-the-badge&logo=rust)](https://github.com/maslina524/lazyfetch/blob/main/src/main.rs)
[![No deps](https://img.shields.io/badge/Fully-no%20deps-green?style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48c3ZnIGZpbGw9IiNmZmZmZmYiIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDE5MjAgMTkyMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cGF0aCBkPSJNMjEzLjMzMyA5NjBjMC0xNjcuMzYgNTYtMzIxLjcwNyAxNDkuNDQtNDQ2LjRMMTQwNi40IDE1NTcuMjI3Yy0xMjQuNjkzIDkzLjQ0LTI3OS4wNCAxNDkuNDQtNDQ2LjQgMTQ5LjQ0LTQxMS42MjcgMC03NDYuNjY3LTMzNS4wNC03NDYuNjY3LTc0Ni42NjdtMTQ5My4zMzQgMGMwIDE2Ny4zNi01NiAzMjEuNzA3LTE0OS40NCA0NDYuNEw1MTMuNiAzNjIuNzczYzEyNC42OTMtOTMuNDQgMjc5LjA0LTE0OS40NCA0NDYuNC0xNDkuNDQgNDExLjYyNyAwIDc0Ni42NjcgMzM1LjA0IDc0Ni42NjcgNzQ2LjY2N005NjAgMEM0MjkuNzYgMCAwIDQyOS43NiAwIDk2MHM0MjkuNzYgOTYwIDk2MCA5NjAgOTYwLTQyOS43NiA5NjAtOTYwUzE0OTAuMjQgMCA5NjAgMCIgZmlsbC1ydWxlPSJldmVub2RkIi8+PC9zdmc+)](https://github.com/maslina524/lazyfetch/blob/main/Cargo.toml)
[![Ru README](https://img.shields.io/badge/En-README-red?style=for-the-badge&logo=readme&logoColor=ffffff)](README.md)

**Lazyfetch** — инструмент в стиле neofetch для красивого вывода системной информации с гибкой настройкой. Написан полностью на Rust с атрибутом `#![no_std]` и без зависимостей, кроме `core` и `alloc`.

> [!NOTE]
>
> Проект протестирован на Windows 11 (x86_64), Debian 13.7 (x86_64) и Android 16 (aarch64).

> [!WARNING]
>
> На данный момент поддерживаются только видеокарты Nvidia. Если у вас GPU другого производителя, модуль GPU может работать некорректно.

<table>
  <tr>
    <td width="60%">
      <img src="screenshots/custom-preset.png" width="100%" alt="Example 1" />
    </td>
    <td width="30%">
      <img src="screenshots/debian-logo.png" width="100%" alt="Example 2" />
      <br />
      <img src="screenshots/fastfetch-preset.png" width="100%" alt="Example 3" />
    </td>
  </tr>
</table>

## О проекте

Проект создан на основе [Fastfetch](https://github.com/fastfetch-cli/fastfetch) и совместим с его конфигами — для экспериментов и улучшения оригинального проекта.

Полностью написан на чистом Rust с атрибутом `#![no_std]` и без зависимостей, кроме `core` и `alloc`. Проходит [clippy](https://github.com/rust-lang/rust-clippy) с `#![deny(clippy::all)]`.

Поддерживаемые платформы: Linux, Windows и Android.

## Особенности

### Кэширование

Lazyfetch использует два уровня кэша:

- **HTTP-кэш** — для модулей, которым нужен доступ к сети (`weather`, `publicip`).
  Обновляется раз в час.
- **Сессионный кэш** — для дорогих в вычислении полей, которые не меняются
  между перезагрузками (имя `GPU`, информация о железе). Считается один раз
  за сессию и переиспользуется в последующих запусках.

**Эффект**: первый запуск ~500 мс (NVML ~50 мс), последующие — ~25 мс.

### Lua-скрипты

Проект использует Lua для встраивания в конфиги:
```json
{
  "type": "title",
  "format": "lua: return 'Hello ' .. (...).hostNameColored .. '!'"
}
```

## Comparison

| Feature      | Fastfetch | Lazyfetch           |
| ------------ | --------- | ------------------- |
| Язык         | C         | Rust                |
| Зависимости  | Yes       | No                  |
| Размер       | ~10Мб     | ~800Кб (inline lua) |
| Android      | Тяжело    | Готовый бинарь      |
| Модули       | 70+       | 20+                 |
| Кеширование  | Нет       | Есть (~1Кб)         |
| macOS/BSD    | Есть      | Не поддерживается   |

Lazyfetch также совместим с конфигами Fastfetch.

## Использование

Запуск сразу после установки:
```bash
$ lazyfetch
```

Указать свой JSON-конфиг:
```bash
$ lazyfetch --config path/to/config.jsonc
$ lazyfetch -c path/to/config.jsonc
```

Выбрать свой логотип:
```bash
$ lazyfetch --logo debian        # Built-in
$ lazyfetch -l debian            # Built-in
$ lazyfetch -l path/to/logo.txt  # Custom ASCII logo
$ lazyfetch -l path/to/image.png # Png image
```

## Установка

Lazyfetch доступен для `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` и `aarch64-linux-android`. Если вашей ОС или архитектуры нет в списке — оставьте запрос в Issues.

### Готовый бинарь

Зайдите в Releases и скачайте готовый бинарь для вашей ОС и архитектуры. Дополнительных зависимостей для работы программы не требуется.

### Сборка из исходников

Сборка из исходников, если у вас установлен cargo:

```bash
$ cargo install --git https://github.com/maslina524/lazyfetch lazyfetch
```

<img src="images/jarvis.gif" width="50%" />
