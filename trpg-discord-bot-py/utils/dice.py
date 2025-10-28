import re
import random
from typing import List, Optional, Tuple
from models.types import DiceRoll, RollResult
from utils.config import GuildConfig


def parse_dice_expr(expr: str, rules: GuildConfig) -> DiceRoll:
    """
    解析骰子表達式，比如 "2d6+1" 或 "d20>=15"
    """
    expr = expr.strip()
    # 正則表達式匹配骰子表達式，如: 2d6+1, d20, 1d10>=15
    pattern = r"^(\d*)d(\d+)([+-]\d+)?(?:\s*(>=|<=|>|<)\s*(\d+))?$"
    match = re.match(pattern, expr)
    
    if not match:
        raise ValueError("無效的骰子表達式格式")
    
    count_str = match.group(1)
    count = int(count_str) if count_str else 1
    
    if count == 0:
        raise ValueError("骰子數量必須至少為1")
    
    if count > rules.dnd_max_dice_count:
        raise ValueError(f"骰子數量過多 (最多 {rules.dnd_max_dice_count})")
    
    sides = int(match.group(2))
    
    if sides < 2:
        raise ValueError("骰子面數必須至少為2")
    
    if sides > rules.dnd_max_dice_sides:
        raise ValueError(f"骰子面數過多 (最多 {rules.dnd_max_dice_sides})")
    
    modifier_match = match.group(3)
    modifier = int(modifier_match) if modifier_match else 0
    
    op_match = match.group(4)
    value_match = match.group(5)
    comparison = (op_match, int(value_match)) if op_match and value_match else None
    
    return DiceRoll(
        count=count,
        sides=sides,
        modifier=modifier,
        comparison=comparison
    )


def roll_single_dice(sides: int) -> int:
    """擲單個骰子"""
    return random.randint(1, sides)


def roll_dice(dice: DiceRoll) -> RollResult:
    """擲骰子並返回結果"""
    rolls = [roll_single_dice(dice.sides) for _ in range(dice.count)]
    
    total = sum(rolls) + dice.modifier
    
    # 檢查大成功/大失敗 (D&D規則，僅適用於d20)
    is_critical_success = dice.sides == 20 and 20 in rolls
    is_critical_fail = dice.sides == 20 and 1 in rolls
    
    # 評估比較運算（如果存在）
    comparison_result = None
    if dice.comparison:
        op, value = dice.comparison
        if op == ">=":
            comparison_result = total >= value
        elif op == ">":
            comparison_result = total > value
        elif op == "<=":
            comparison_result = total <= value
        elif op == "<":
            comparison_result = total < value
    
    return RollResult(
        dice_expr=format_dice_expr(dice),
        rolls=rolls,
        modifier=dice.modifier,
        total=total,
        is_critical_success=is_critical_success,
        is_critical_fail=is_critical_fail,
        comparison_result=comparison_result
    )


def format_dice_expr(dice: DiceRoll) -> str:
    """格式化骰子表達式"""
    if dice.modifier == 0:
        return f"{dice.count}d{dice.sides}"
    else:
        sign = "+" if dice.modifier >= 0 else ""
        return f"{dice.count}d{dice.sides}{sign}{dice.modifier}"


def roll_multiple_dice(expr: str, max_rolls: int, rules: GuildConfig) -> List[RollResult]:
    """
    解析並擲多個骰子表達式（用於連続擲骰）
    """
    # 檢查是否為多次擲骰格式，例如 "+3 d4"
    pattern = r"^(?:\+)?(\d+)\s+(.+)$"
    match = re.match(pattern, expr.strip())
    
    if match:
        count = int(match.group(1))
        
        if count == 0:
            raise ValueError("擲骰次數必須至少為1")
        
        if count > max_rolls:
            raise ValueError(f"擲骰次數過多 (最多 {max_rolls})")
        
        dice_expr = match.group(2).strip()
        parsed_dice = parse_dice_expr(dice_expr, rules)
        
        return [roll_dice(parsed_dice) for _ in range(count)]
    else:
        # 單次擲骰
        parsed_dice = parse_dice_expr(expr, rules)
        return [roll_dice(parsed_dice)]


def format_roll_result(result: RollResult) -> str:
    """格式化骰子結果"""
    rolls_str = " + ".join(map(str, result.rolls))
    
    if result.modifier != 0:
        total_with_mod = f"({rolls_str}) + {result.modifier} = {result.total}"
    else:
        total_with_mod = f"{rolls_str} = {result.total}"
    
    crit_info = ""
    if result.is_critical_success:
        crit_info = " ✨ 大成功!"
    elif result.is_critical_fail:
        crit_info = " 💥 大失敗!"
    
    comparison_info = ""
    if result.comparison_result is True:
        comparison_info = "✅ 成功 "
    elif result.comparison_result is False:
        comparison_info = "❌ 失敗 "
    
    return f"🎲 D&D 擲骰: {result.dice_expr} = {total_with_mod}{crit_info}{comparison_info}"


def format_multiple_roll_results(results: List[RollResult]) -> str:
    """格式化多個骰子結果"""
    output = "🎲 連續擲骰結果:\n"
    
    for i, result in enumerate(results):
        rolls_str = " + ".join(map(str, result.rolls))
        
        if result.modifier != 0:
            total_with_mod = f"({rolls_str}) + {result.modifier} = {result.total}"
        else:
            total_with_mod = f"{rolls_str} = {result.total}"
        
        output += f"{i + 1}. {result.dice_expr} = {total_with_mod}\n"
    
    return output