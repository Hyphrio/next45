-- Migration number: 0005 	 2025-03-31T11:31:59.812Z

CREATE TABLE temp_table AS
SELECT * FROM Attempts;

INSERT INTO temp_table
SELECT * FROM Attempts;

DROP TABLE Attempts;

ALTER TABLE temp_table RENAME TO Attempts;
