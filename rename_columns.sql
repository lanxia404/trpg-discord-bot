-- 重命名 base_settings.db 中的資料表欄位

-- 1. 重命名 藥水 表的欄位
-- 原始欄位: "__", "___1", "___2", "___3"
-- 重命名為: "名稱", "材料", "效果", "等級"
ALTER TABLE "藥水" RENAME COLUMN "__" TO "名稱";
ALTER TABLE "藥水" RENAME COLUMN "___1" TO "材料";
ALTER TABLE "藥水" RENAME COLUMN "___2" TO "效果";
ALTER TABLE "藥水" RENAME COLUMN "___3" TO "等級";

-- 2. 重命名 忍術_手勢 表的欄位
-- 原始欄位: "__", "___1"
-- 重命名為: "手勢名稱", "對應忍術"
ALTER TABLE "忍術_手勢" RENAME COLUMN "__" TO "手勢名稱";
ALTER TABLE "忍術_手勢" RENAME COLUMN "___1" TO "對應忍術";

-- 3. 重命名 忍術_忍術 表的欄位
-- 原始欄位: "____", "_____1", "_____2", "__", "___1"
-- 重命名為: "忍術名稱", "屬性", "等級", "消耗查克拉", "效果描述"
ALTER TABLE "忍術_忍術" RENAME COLUMN "____" TO "忍術名稱";
ALTER TABLE "忍術_忍術" RENAME COLUMN "_____1" TO "屬性";
ALTER TABLE "忍術_忍術" RENAME COLUMN "_____2" TO "等級";
ALTER TABLE "忍術_忍術" RENAME COLUMN "__" TO "消耗查克拉";
ALTER TABLE "忍術_忍術" RENAME COLUMN "___1" TO "效果描述";

-- 4. 重命名 異常狀態 表的欄位
-- 原始欄位: "__", "____", "_____1"
-- 重命名為: "狀態名稱", "持續時間", "效果描述"
ALTER TABLE "異常狀態" RENAME COLUMN "__" TO "狀態名稱";
ALTER TABLE "異常狀態" RENAME COLUMN "____" TO "持續時間";
ALTER TABLE "異常狀態" RENAME COLUMN "_____1" TO "效果描述";

-- 5. 重命名 盧恩奧術 表的欄位
-- 原始欄位: "___", "__"
-- 重命名為: "盧恩名稱", "效果"
ALTER TABLE "盧恩奧術" RENAME COLUMN "___" TO "盧恩名稱";
ALTER TABLE "盧恩奧術" RENAME COLUMN "__" TO "效果";

-- 注意：元素反應表的欄位名稱 ("row_header", "col_header", "value") 已經是描述性的，所以不需要重命名