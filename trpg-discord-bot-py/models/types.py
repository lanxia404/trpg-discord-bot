from dataclasses import dataclass
from typing import List, Optional, Tuple


@dataclass
class RollResult:
    """骰子結果"""
    dice_expr: str
    rolls: List[int]
    modifier: int
    total: int
    is_critical_success: bool = False
    is_critical_fail: bool = False
    comparison_result: Optional[bool] = None  # Some(true) for success, Some(false) for failure, None for no comparison


@dataclass
class DiceRoll:
    """骰子配置"""
    count: int
    sides: int
    modifier: int
    comparison: Optional[Tuple[str, int]] = None  # (operator, value) e.g. (">=", 15)