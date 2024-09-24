# SQLX-CLI迁移步骤

## 1. 安装依赖

```bash
cargo install sqlx-cli
```

## 2. 创建迁移

```bash
sqlx migrate add create_user
```

## 3. 运行迁移

```bash
sqlx migrate run
```

## 5. 回滚迁移

```bash
sqlx migrate revert
```