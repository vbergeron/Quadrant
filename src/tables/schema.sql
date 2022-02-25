CREATE TABLE `block` (
    `height`   INTEGER,
    `time`     TEXT,
    `hash`     TEXT,
    `proposer` TEXT,
    PRIMARY KEY (`height`)
);

CREATE UNIQUE INDEX `idx_block_time` ON `block`(`time`);
CREATE UNIQUE INDEX `idx_block_hash` ON `block`(`hash`);

CREATE TABLE `tx` (
    `block` INTEGER REFERENCES `block`(`height`),
    `idx`   INTEGER,
    `hash`  TEXT,
    PRIMARY KEY (`block`, `idx`)
);

CREATE UNIQUE INDEX `idx_tx_hash` ON `tx`(`hash`);

CREATE TABLE `msg` (
    `block` INTEGER REFERENCES `block`(`height`),
    `tx`    INTEGER REFERENCES `tx`(`idx`),
    `idx`   INTEGER,
    `tag`   TEXT,
    `data`  BLOB,
    PRIMARY KEY (`block`,`tx`, `idx`)
);

CREATE TABLE `address_msg` (
    `address` TEXT,
    `block`   INTEGER REFERENCES `block`(`height`),
    `tx`      INTEGER REFERENCES `tx`(`idx`),
    `msg`     INTEGER REFERENCES `msg`(`idx`),
    `id`      ROWID
);
