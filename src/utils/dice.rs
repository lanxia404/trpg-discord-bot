use crate::models::types::{DiceRoll, DnDRules, RollResult};
use rand::Rng;
use regex::Regex;

/// Parse a dice expression like "2d6+1" or "d20>=15"
pub fn parse_dice_expr(expr: &str, rules: &DnDRules) -> Result<DiceRoll, String> {
    let expr = expr.trim();
    let re = Regex::new(r"^(\d*)d(\d+)([+\-]\d+)?(?:\s*(>=|<=|>|<)\s*(\d+))?$")
        .map_err(|_| "Invalid regex pattern")?;

    let captures = re
        .captures(expr)
        .ok_or_else(|| "Invalid dice expression format".to_string())?;

    let count_str = captures.get(1).map_or("1", |m| m.as_str());
    let count = if count_str.is_empty() {
        1
    } else {
        count_str
            .parse::<u8>()
            .map_err(|_| "Invalid dice count".to_string())?
    };

    if count == 0 {
        return Err("Dice count must be at least 1".to_string());
    }

    if count > rules.max_dice_count {
        return Err(format!("Too many dice (max {})", rules.max_dice_count));
    }

    let sides = captures
        .get(2)
        .ok_or_else(|| "Missing dice sides".to_string())?
        .as_str()
        .parse::<u16>()
        .map_err(|_| "Invalid dice sides".to_string())?;

    if sides < 2 {
        return Err("Dice must have at least 2 sides".to_string());
    }

    if sides > rules.max_dice_sides {
        return Err(format!(
            "Dice has too many sides (max {})",
            rules.max_dice_sides
        ));
    }

    let modifier = captures
        .get(3)
        .map(|m| m.as_str())
        .unwrap_or("0")
        .parse::<i32>()
        .map_err(|_| "Invalid modifier".to_string())?;

    let comparison = if let Some(op_match) = captures.get(4) {
        let op = op_match.as_str().to_string();
        let value = captures
            .get(5)
            .ok_or_else(|| "Missing comparison value".to_string())?
            .as_str()
            .parse::<i32>()
            .map_err(|_| "Invalid comparison value".to_string())?;
        Some((op, value))
    } else {
        None
    };

    Ok(DiceRoll {
        count,
        sides,
        modifier,
        comparison,
    })
}

/// Roll a single dice with given sides
pub fn roll_single_dice(sides: u16) -> u16 {
    rand::thread_rng().gen_range(1..=sides)
}

/// Roll multiple dice and return results
pub fn roll_dice(dice: &DiceRoll) -> RollResult {
    let mut rolls = Vec::new();

    for _ in 0..dice.count {
        rolls.push(roll_single_dice(dice.sides));
    }

    let total = rolls.iter().map(|&r| r as i32).sum::<i32>() + dice.modifier;

    // Check for critical success/fail
    let is_critical_success = dice.sides == 20 && rolls.contains(&20);
    let is_critical_fail = dice.sides == 20 && rolls.contains(&1);

    // Evaluate comparison if present
    let comparison_result = match &dice.comparison {
        Some((op, value)) => match op.as_str() {
            ">=" => Some(total >= *value),
            ">" => Some(total > *value),
            "<=" => Some(total <= *value),
            "<" => Some(total < *value),
            _ => None,
        },
        None => None,
    };

    RollResult {
        dice_expr: format_dice_expr(dice),
        rolls,
        modifier: dice.modifier,
        total,
        is_critical_success,
        is_critical_fail,
        comparison_result,
    }
}

fn format_dice_expr(dice: &DiceRoll) -> String {
    let modifier = if dice.modifier == 0 {
        String::new()
    } else if dice.modifier > 0 {
        format!("+{}", dice.modifier)
    } else {
        dice.modifier.to_string()
    };

    format!("{}d{}{}", dice.count, dice.sides, modifier)
}

/// Parse and roll multiple dice expressions (for consecutive rolls)
pub fn roll_multiple_dice(
    expr: &str,
    max_rolls: u8,
    rules: &DnDRules,
) -> Result<Vec<RollResult>, String> {
    let re = Regex::new(r"^(?:\+)?(\d+)\s+(.+)$").map_err(|_| "Invalid regex pattern")?;

    if let Some(captures) = re.captures(expr.trim()) {
        let count = captures
            .get(1)
            .ok_or_else(|| "Missing roll count".to_string())?
            .as_str()
            .parse::<u8>()
            .map_err(|_| "Invalid roll count".to_string())?;

        if count == 0 {
            return Err("Roll count must be at least 1".to_string());
        }

        if count > max_rolls {
            return Err(format!("Too many rolls (max {})", max_rolls));
        }

        let expr = captures
            .get(2)
            .ok_or_else(|| "Missing dice expression".to_string())?
            .as_str()
            .trim();

        let parsed_dice = parse_dice_expr(expr, rules)?;

        let mut results = Vec::new();
        for _ in 0..count {
            results.push(roll_dice(&parsed_dice));
        }

        Ok(results)
    } else {
        let parsed_dice = parse_dice_expr(expr, rules)?;
        Ok(vec![roll_dice(&parsed_dice)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dice_expr() {
        let rules = DnDRules::default();
        let dice = parse_dice_expr("2d6+1", &rules).unwrap();
        assert_eq!(dice.count, 2);
        assert_eq!(dice.sides, 6);
        assert_eq!(dice.modifier, 1);
    }

    #[test]
    fn test_parse_dice_expr_without_modifier() {
        let rules = DnDRules::default();
        let dice = parse_dice_expr("d20", &rules).unwrap();
        assert_eq!(dice.count, 1);
        assert_eq!(dice.sides, 20);
        assert_eq!(dice.modifier, 0);
    }

    #[test]
    fn test_roll_dice() {
        let dice = DiceRoll {
            count: 1,
            sides: 6,
            modifier: 0,
            comparison: None,
        };

        let result = roll_dice(&dice);
        assert!(result.rolls[0] >= 1 && result.rolls[0] <= 6);
        assert_eq!(result.total, result.rolls[0] as i32);
    }

    #[test]
    fn test_roll_multiple_dice() {
        let rules = DnDRules::default();
        let results = roll_multiple_dice("+3 d4", rules.max_dice_count, &rules).unwrap();
        assert_eq!(results.len(), 3);
    }
}
