-- Add migration script here
BEGIN;
    UPDATE `zero2prod`.`subscriptions` SET `status` = 'confirmed' WHERE `status` IS NULL;

    ALTER TABLE `zero2prod`.`subscriptions` MODIFY `status` VARCHAR(50) NOT NULL;
COMMIT;
