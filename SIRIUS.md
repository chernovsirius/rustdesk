# SiriusDesk — сборка с зашитым сервером

## Секреты GitHub

`Settings → Secrets and variables → Actions → New repository secret`:

| Секрет                | Пример                      | Обязателен |
|-----------------------|-----------------------------|------------|
| `SIRIUS_ID_SERVER`    | `desk.it-sirius.ru`         | да         |
| `SIRIUS_API_SERVER`   | `https://desk.it-sirius.ru` | да, если нужна статистика/CRM |
| `SIRIUS_KEY`          | содержимое `id_ed25519.pub` | да         |
| `SIRIUS_RELAY_SERVER` | `desk.it-sirius.ru:21117`   | нет (по умолчанию берётся из ID-сервера) |

Сборка: `Actions → Flutter Tag Build → Run workflow` (или пуш тега `vX.Y.Z`).
Значения подставляются при компиляции (`option_env!` в `src/sirius.rs`). Если
`SIRIUS_ID_SERVER` не задан, клиент ведёт себя как обычный RustDesk.

Локальная сборка: те же переменные окружения перед `python3 build.py --flutter`.

## Как это работает у клиента

* **Новая установка** — при первом запуске сервис сам прописывает сервер Сириус,
  вводить ничего не нужно.
* **Уже настроен свой (корпоративный) сервер** — настройки не трогаются.
* **Настройки → Сеть → ID/Ретранслятор** — выпадающий список:
  * *Сервер Сириус* — параметры из сборки, поля скрыты;
  * *Свои настройки* — можно ввести корпоративный сервер или оставить поля
    пустыми, тогда используются общедоступные серверы RustDesk.
* Проверка обновлений с rustdesk.com отключена, чтобы официальный релиз не
  заменил брендированную сборку.

## Брендинг

* Цвета: `MyTheme.siriusBlue` / `MyTheme.siriusBlueLight` в `flutter/lib/common.dart`.
* Логотипы и иконки: `res/` (исходники), `flutter/assets/icon.svg`,
  `flutter/windows/runner/resources/app_icon.ico`, иконки Android/iOS сгенерированы
  из `res/icon.png`.
* Иконка macOS (`flutter/macos/Runner/AppIcon.icns`) пока прежняя.

## Обновление из апстрима

```sh
git remote add upstream https://github.com/rustdesk/rustdesk.git  # один раз
git fetch upstream master
git merge upstream/master
git submodule update --init --recursive
```
