import random
from models.types import RollResult
from utils.config import GuildConfig


def roll_coc(skill_value: int, rules: GuildConfig) -> RollResult:
    """
    擲Call of Cthulhu 7th edition骰子
    """
    roll = random.randint(1, 100)
    
    success_level = determine_success_level(roll, skill_value, rules)
    
    is_critical_success = roll == rules.coc_critical_success
    is_critical_fail = is_critical_failure(roll, skill_value, rules)
    
    # 判定結果：success_level <= 4 表示成功，其他表示失敗
    comparison_result = success_level <= 4
    
    return RollResult(
        dice_expr=f"d100<={skill_value}",
        rolls=[roll],
        modifier=0,
        total=roll,
        is_critical_success=is_critical_success,
        is_critical_fail=is_critical_fail,
        comparison_result=comparison_result
    )


def roll_coc_multi(skill_value: int, times: int, rules: GuildConfig) -> list[RollResult]:
    """
    多次擲Call of Cthulhu 7th edition骰子
    """
    count = max(1, times)
    return [roll_coc(skill_value, rules) for _ in range(count)]


def determine_success_level(roll: int, skill_value: int, rules: GuildConfig) -> int:
    """
    根據CoC 7e規則確定成功等級
    返回: 1=大成功, 2=極限成功, 3=困難成功, 4=普通成功, 5=失敗, 6=大失敗
    """
    if roll == rules.coc_critical_success:
        return 1  # 大成功
    
    if is_critical_failure(roll, skill_value, rules):
        return 6  # 大失敗
    
    hard_success_threshold = skill_value / rules.coc_skill_divisor_hard
    extreme_success_threshold = skill_value / rules.coc_skill_divisor_extreme
    
    if roll == 100 or roll <= extreme_success_threshold:
        return 2  # 極限成功
    elif roll <= hard_success_threshold:
        return 3  # 困難成功
    elif roll <= skill_value:
        return 4  # 普通成功
    else:
        return 5  # 失敗


def is_critical_failure(roll: int, skill_value: int, rules: GuildConfig) -> bool:
    """
    根據CoC 7e規則檢查是否為大失敗
    """
    if skill_value < 50:
        # 對於低於50%的技能，96-100是大失敗
        return roll >= 96
    else:
        # 對於50%或更高的技能，只有100是大失敗
        return roll == rules.coc_critical_fail


def format_success_level(level: int) -> str:
    """
    將成功等級格式化為字符串
    """
    level_map = {
        1: "大成功 (Critical Success)",
        2: "極限成功 (Extreme Success)",
        3: "困難成功 (Hard Success)",
        4: "普通成功 (Regular Success)",
        5: "失敗 (Failure)",
        6: "大失敗 (Critical Failure)"
    }
    return level_map.get(level, "未知 (Unknown)")


def format_coc_result(result: RollResult, skill_value: int) -> str:
    """
    格式化CoC結果
    """
    success_level = determine_success_level(result.total, skill_value, GuildConfig())
    success_text = format_success_level(success_level)
    
    crit_info = ""
    if result.is_critical_success:
        crit_info = " ✨ 大成功!"
    elif result.is_critical_fail:
        crit_info = " 💥 大失敗!"
    
    return f"技能值: {skill_value}\n骰子結果: {result.rolls[0]}\n判定結果: {success_text}{crit_info}"


def format_coc_multi_results(results: list[RollResult], skill_value: int) -> str:
    """
    格式化CoC多次擲骰結果
    """
    message = f"連續擲骰次數: {len(results)}\n技能值: {skill_value}\n"
    
    for i, result in enumerate(results):
        success_level = determine_success_level(result.total, skill_value, GuildConfig())
        success_text = format_success_level(success_level)
        
        crit = " ✨" if result.is_critical_success else " 💥" if result.is_critical_fail else ""
        status = " ✅" if result.comparison_result else " ❌" if result.comparison_result is False else ""
        
        message += f"{i + 1}. {result.rolls[0]} → {success_text}{crit}{status}\n"
    
    return message