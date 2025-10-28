import sqlite3
import os
from typing import Optional, List, Tuple
from dataclasses import dataclass


@dataclass
class Skill:
    guild_id: int
    user_id: int
    name: str
    normalized_name: str
    skill_type: str
    level: str
    effect: str


class SkillsDB:
    """技能數據庫類"""
    def __init__(self, db_path: str = "skills.db"):
        self.db_path = db_path
        self.init_db()
    
    def init_db(self):
        """初始化數據庫"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # 創建技能表
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS skills (
                guild_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                normalized_name TEXT NOT NULL,
                skill_type TEXT NOT NULL,
                level TEXT NOT NULL,
                effect TEXT NOT NULL,
                UNIQUE(guild_id, user_id, normalized_name)
            )
        ''')
        
        # 檢查並升級表結構
        cursor.execute("PRAGMA table_info(skills)")
        columns = [column[1] for column in cursor.fetchall()]
        
        # 添加缺少的欄位
        if 'skill_type' not in columns:
            cursor.execute("ALTER TABLE skills ADD COLUMN skill_type TEXT NOT NULL DEFAULT ''")
        if 'level' not in columns:
            cursor.execute("ALTER TABLE skills ADD COLUMN level TEXT NOT NULL DEFAULT ''")
        if 'effect' not in columns:
            cursor.execute("ALTER TABLE skills ADD COLUMN effect TEXT NOT NULL DEFAULT ''")
        
        conn.commit()
        conn.close()
    
    def add_skill(self, guild_id: int, user_id: int, name: str, skill_type: str, level: str, effect: str):
        """添加或更新技能"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        normalized = name.lower()
        
        cursor.execute('''
            INSERT INTO skills (guild_id, user_id, name, normalized_name, skill_type, level, effect)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(guild_id, user_id, normalized_name)
            DO UPDATE SET name=excluded.name, skill_type=excluded.skill_type, level=excluded.level, effect=excluded.effect
        ''', (guild_id, user_id, name, normalized, skill_type, level, effect))
        
        conn.commit()
        conn.close()
    
    def find_skill_for_user(self, guild_id: int, user_id: int, name: str) -> Optional[Skill]:
        """為用戶查找技能"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        normalized = name.lower()
        pattern = f"%{normalized}%"
        
        cursor.execute('''
            SELECT name, normalized_name, user_id, skill_type, level, effect
            FROM skills
            WHERE guild_id = ? AND user_id = ? AND normalized_name LIKE ?
            ORDER BY CASE WHEN normalized_name = ? THEN 0 ELSE 1 END,
                     ABS(LENGTH(normalized_name) - LENGTH(?)),
                     normalized_name
            LIMIT 1
        ''', (guild_id, user_id, pattern, normalized, normalized))
        
        row = cursor.fetchone()
        conn.close()
        
        if row:
            return Skill(
                guild_id=guild_id,
                user_id=row[2],  # user_id
                name=row[0],     # name
                normalized_name=row[1],  # normalized_name
                skill_type=row[3],       # skill_type
                level=row[4],            # level
                effect=row[5]            # effect
            )
        return None
    
    def find_skill_in_guild(self, guild_id: int, name: str) -> Optional[Skill]:
        """在公會中查找技能"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        normalized = name.lower()
        pattern = f"%{normalized}%"
        
        cursor.execute('''
            SELECT name, normalized_name, user_id, skill_type, level, effect
            FROM skills
            WHERE guild_id = ? AND normalized_name LIKE ?
            ORDER BY CASE WHEN normalized_name = ? THEN 0 ELSE 1 END,
                     ABS(LENGTH(normalized_name) - LENGTH(?)),
                     normalized_name
            LIMIT 1
        ''', (guild_id, pattern, normalized, normalized))
        
        row = cursor.fetchone()
        conn.close()
        
        if row:
            return Skill(
                guild_id=guild_id,
                user_id=row[2],  # user_id
                name=row[0],     # name
                normalized_name=row[1],  # normalized_name
                skill_type=row[3],       # skill_type
                level=row[4],            # level
                effect=row[5]            # effect
            )
        return None
    
    def delete_skill(self, guild_id: int, owner_id: int, normalized_name: str):
        """刪除技能"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            DELETE FROM skills
            WHERE guild_id = ? AND user_id = ? AND normalized_name = ?
        ''', (guild_id, owner_id, normalized_name))
        
        conn.commit()
        conn.close()
    
    def get_all_skills_for_user(self, guild_id: int, user_id: int) -> List[Skill]:
        """獲取用戶的所有技能"""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        cursor.execute('''
            SELECT name, normalized_name, user_id, skill_type, level, effect
            FROM skills
            WHERE guild_id = ? AND user_id = ?
        ''', (guild_id, user_id))
        
        rows = cursor.fetchall()
        conn.close()
        
        skills = []
        for row in rows:
            skills.append(Skill(
                guild_id=guild_id,
                user_id=row[2],
                name=row[0],
                normalized_name=row[1],
                skill_type=row[3],
                level=row[4],
                effect=row[5]
            ))
        
        return skills