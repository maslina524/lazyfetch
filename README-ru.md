# Lazyfetch

[![GitHub license](https://img.shields.io/github/license/fastfetch-cli/fastfetch?style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48c3ZnIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIG9wYWNpdHk9IjAuMSIgZD0iTTEyIDE3SDdDNS44OTU0MyAxNyA1IDE2LjEwNDYgNSAxNVY1QzUgMy44OTU0MyA1Ljg5NTQzIDMgNyAzSDE2QzE3LjEwNDYgMyAxOCAzLjg5NTQzIDE4IDVWMTlDMTggMjAuMTA0NiAxNy4xMDQ2IDIxIDE2IDIxQzE0Ljg5NTQgMjEgMTQgMjAuMTA0NiAxNCAxOUMxNCAxNy44OTU0IDEzLjEwNDYgMTcgMTIgMTdaIiBmaWxsPSIjZmZmZmZmIi8+PHBhdGggZD0iTTE5IDNIOVYzQzcuMTE0MzggMyA2LjE3MTU3IDMgNS41ODU3OSAzLjU4NTc5QzUgNC4xNzE1NyA1IDUuMTE0MzggNSA3VjEwLjVWMTciIHN0cm9rZT0iI2ZmZmZmZiIgc3Ryb2tlLXdpZHRoPSIyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz48cGF0aCBkPSJNMTQgMTdWMTlDMTQgMjAuMTA0NiAxNC44OTU0IDIxIDE2IDIxVjIxQzE3LjEwNDYgMjEgMTggMjAuMTA0NiAxOCAxOVY5VjQuNUMxOCAzLjY3MTU3IDE4LjY3MTYgMyAxOS41IDNWM0MyMC4zMjg0IDMgMjEgMy42NzE1NyAyMSA0LjVWNC41QzIxIDUuMzI4NDMgMjAuMzI4NCA2IDE5LjUgNkgxOC41IiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+PHBhdGggZD0iTTE2IDIxSDVDMy44OTU0MyAyMSAzIDIwLjEwNDYgMyAxOVYxOUMzIDE3Ljg5NTQgMy44OTU0MyAxNyA1IDE3SDE0IiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+PHBhdGggZD0iTTkgN0gxNCIgc3Ryb2tlPSIjZmZmZmZmIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIvPjxwYXRoIGQ9Ik05IDExSDE0IiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIi8+PC9zdmc+)](https://github.com/maslina524/lazyfetch/blob/main/LICENSE)
[![GitHub top language](https://img.shields.io/github/languages/top/maslina524/lazyfetch?style=for-the-badge&logo=rust&label=Rust)](https://github.com/maslina524/lazyfetch/blob/main/Cargo.toml)
[![GitHub commit activity](https://img.shields.io/github/commit-activity/m/maslina524/lazyfetch?style=for-the-badge&logo=github)](https://github.com/maslina524/lazyfetch/commits)  
[![No std](https://img.shields.io/badge/Built_with-%23!%5Bno__std%5D-blue?style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48IS0tIFVwbG9hZGVkIHRvOiBTVkcgUmVwbywgd3d3LnN2Z3JlcG8uY29tLCBHZW5lcmF0b3I6IFNWRyBSZXBvIE1peGVyIFRvb2xzIC0tPgo8c3ZnIGZpbGw9IiNmZmZmZmYiIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDI0IDI0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik0xNC4yNSw4SDkuNzVBMS43NTIsMS43NTIsMCwwLDAsOCw5Ljc1djQuNUExLjc1MiwxLjc1MiwwLDAsMCw5Ljc1LDE2aDQuNUExLjc1MiwxLjc1MiwwLDAsMCwxNiwxNC4yNVY5Ljc1QTEuNzUyLDEuNzUyLDAsMCwwLDE0LjI1LDhaTTE0LDE0SDEwVjEwaDRabTgtNWExLDEsMCwwLDAsMC0ySDIwVjYuNzVBMi43NTIsMi43NTIsMCwwLDAsMTcuMjUsNEgxN1YyYTEsMSwwLDAsMC0yLDBWNEgxM1YyYTEsMSwwLDAsMC0yLDBWNEg5VjJBMSwxLDAsMCwwLDcsMlY0SDYuNzVBMi43NTIsMi43NTIsMCwwLDAsNCw2Ljc1VjdIMkExLDEsMCwwLDAsMiw5SDR2MkgyYTEsMSwwLDAsMCwwLDJINHYySDJhMSwxLDAsMCwwLDAsMkg0di4yNUEyLjc1MiwyLjc1MiwwLDAsMCw2Ljc1LDIwSDd2MmExLDEsMCwwLDAsMiwwVjIwaDJ2MmExLDEsMCwwLDAsMiwwVjIwaDJ2MmExLDEsMCwwLDAsMiwwVjIwaC4yNUEyLjc1MiwyLjc1MiwwLDAsMCwyMCwxNy4yNVYxN2gyYTEsMSwwLDAsMCwwLTJIMjBWMTNoMmExLDEsMCwwLDAsMC0ySDIwVjlabS00LDguMjVhLjc1MS43NTEsMCwwLDEtLjc1Ljc1SDYuNzVBLjc1MS43NTEsMCwwLDEsNiwxNy4yNVY2Ljc1QS43NTEuNzUxLDAsMCwxLDYuNzUsNmgxMC41YS43NTEuNzUxLDAsMCwxLC43NS43NVoiLz48L3N2Zz4=)](https://github.com/maslina524/lazyfetch/blob/main/src/main.rs)
[![Clippy](https://img.shields.io/badge/Clippy-%23!%5Bdeny%28clippy::all%29%5D-blue?style=for-the-badge&logo=rust)](https://github.com/maslina524/lazyfetch/blob/main/src/main.rs)
[![No deps](https://img.shields.io/badge/Fully-no%20deps-green?style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48c3ZnIGZpbGw9IiNmZmZmZmYiIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDE5MjAgMTkyMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cGF0aCBkPSJNMjEzLjMzMyA5NjBjMC0xNjcuMzYgNTYtMzIxLjcwNyAxNDkuNDQtNDQ2LjRMMTQwNi40IDE1NTcuMjI3Yy0xMjQuNjkzIDkzLjQ0LTI3OS4wNCAxNDkuNDQtNDQ2LjQgMTQ5LjQ0LTQxMS42MjcgMC03NDYuNjY3LTMzNS4wNC03NDYuNjY3LTc0Ni42NjdtMTQ5My4zMzQgMGMwIDE2Ny4zNi01NiAzMjEuNzA3LTE0OS40NCA0NDYuNEw1MTMuNiAzNjIuNzczYzEyNC42OTMtOTMuNDQgMjc5LjA0LTE0OS40NCA0NDYuNC0xNDkuNDQgNDExLjYyNyAwIDc0Ni42NjcgMzM1LjA0IDc0Ni42NjcgNzQ2LjY2N005NjAgMEM0MjkuNzYgMCAwIDQyOS43NiAwIDk2MHM0MjkuNzYgOTYwIDk2MCA5NjAgOTYwLTQyOS43NiA5NjAtOTYwUzE0OTAuMjQgMCA5NjAgMCIgZmlsbC1ydWxlPSJldmVub2RkIi8+PC9zdmc+)](https://github.com/maslina524/lazyfetch/blob/main/Cargo.toml)
[![En README](https://img.shields.io/badge/En-README-red?style=for-the-badge&logo=readme&logoColor=ffffff)](README.md)

**Lazyfetch** это neofetch-подобная утилита для красивого вывода информации о вашей системе с гибкой кастомизацией, написанная полностью на Rust с атрибутом `#![no_std]` и без зависимостей в таргете, кроме стандартного `alloc`

> [!WARNING]
>
> Проект тестируется на Windows 11 (x86_64), Debian 13.7 (x86_64) и Android 16 (aarch64)

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

Проект создан на основе полной совместимости с [Fastfetch](https://github.com/fastfetch-cli/fastfetch) для эксперимента и улучшения.

Полностью написанный на чистом Rust с `#![no_std]` и без сторонних зависимостей в таргете. Создан с соблюдением строгих [clippy](https://github.com/rust-lang/rust-clippy) линтов.

Разрабатывается только под Linux, Windows и Android.

## Сравнение

| Особенность           | Fastfetch | Lazyfetch             |
| --------------------- | --------- | --------------------- |
| Язык                  | C         | Rust                  |
| Зависимости в таргете | Есть      | Нет                   |
| Размер бинарника      | ~10Мб     | ~800Кб (вместе с lua) |
| Android               | Тяжело    | Готовый бинарник      |

Lazyfetch также совместим с Fastfetch конфигами.

## Установка

Lazyfetch доступен на `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` и `aarch64-linux-android`, если здесь нет вашей ОС или архитектуры, оставьте запрос в Issue.

### Готовый бинарник

Перейдите в Releases и скачайте готовый бинарник для вашей ос и архитектуры, никакие дополнительные зависимости для работы программы не требуются

### Сборка из исходников

Сборка из исходников, если у вас установлен `cargo`:

```bash
$ cargo +nightly install --git https://github.com/maslina524/lazyfetch lazyfetch
```

<img src="images/jarvis.gif" width="75%" />