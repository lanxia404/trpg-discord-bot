use crate::models::types::{DiceRoll, RollResult};
use rand::Rng;
use regex::Regex;


/// Parse a dice expression like "2d6+1" or "d20>=15"
pub fn parse_dice_expr(expr: &str) -> Result<DiceRoll, String> {
    // Handle prefix format like "+10 d20" (for multiple rolls)
    let expr = expr.trim();
    let re = Regex::new(r#"^(\+\d+\s+)?(\d*)d(\d+)([+\-]\d+)?(\s*(>=|<=|>|<)\s*\d+)?$"#)
        .map_err(|_| "Invalid regex pattern")?;
    
    if let Some(captures) = re.captures(expr) {
        let count_str = captures.get(2).map_or("1", |m| m.as_str());
        let count = if count_str.is_empty() {
            1
        } else {
            count_str.parse::<u8>().map_err(|_| "Invalid dice count")?
        };
        
        let sides = captures.get(3).map_or("20", |m| m.as_str()).parse::<u16>()
            .map_err(|_| "Invalid dice sides")?;
        
        let modifier_str = captures.get(4).map(|m| m.as_str()).unwrap_or("0");
        let modifier = if modifier_str.is_empty() {
            0
        } else {
            modifier_str.parse::<i32>().map_err(|_| "Invalid modifier")?
        };
        
        // Parse comparison if present (e.g., ">=15")
        let comparison = if let Some(_) = captures.get(5) {
            let op = captures.get(6).map(|m| m.as_str()).unwrap_or("");
            let value = captures.get(7).map(|m| m.as_str()).unwrap_or("0");
            let value = value.parse::<i32>().map_err(|_| "Invalid comparison value")?;
            Some((op.to_string(), value))
        } else {
            None
        };
        
        // Validate limits
        if count > 50 {
            return Err("Too many dice (max 50)".to_string());
        }
        
        if sides > 1000 {
            return Err("Dice has too many sides (max 1000)".to_string());
        }
        
        Ok(DiceRoll {
            count,
            sides,
            modifier,
            comparison,
        })
    } else {
        Err("Invalid dice expression format".to_string())
    }
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
        Some((op, value)) => {
            match op.as_str() {
                ">=" => Some(total >= *value),
                ">" => Some(total > *value),
                "<=" => Some(total <= *value),
                "<" => Some(total < *value),
                _ => None,
            }
        }
        None => None,
    };
    
    RollResult {
        dice_expr: format!("{}d{}{}", dice.count, dice.sides, 
                          if dice.modifier >= 0 { format!("+{}", dice.modifier) } else { format!("{}", dice.modifier) }),
        rolls,
        modifier: dice.modifier,
        total,
        is_critical_success,
        is_critical_fail,
        comparison_result,
    }
}

/// Parse and roll multiple dice expressions (for consecutive rolls)
pub fn roll_multiple_dice(expr: &str, max_rolls: u8) -> Result<Vec<RollResult>, String> {
    // Check for prefix format like "+10 d20" (multiple rolls)
    let re = Regex::new(r#"^(\+)?(\d+)\s+(.+)"#).map_err(|_| "Invalid regex pattern")?;
    
    if let Some(captures) = re.captures(expr.trim()) {
        let _is_additive = captures.get(1).is_some(); // Check if it's additive (with +)
        let count = captures.get(2).map_or("1", |m| m.as_str()).parse::<u8>()
            .map_err(|_| "Invalid roll count")?;
        
        if count > max_rolls {
            return Err(format!("Too many rolls (max {})", max_rolls));
        }
        
        let expr = captures.get(3).map(|m| m.as_str()).unwrap_or("").trim();
        let parsed_dice = parse_dice_expr(expr)?;
        
        let mut results = Vec::new();
        for _ in 0..count {
            results.push(roll_dice(&parsed_dice));
        }
        
        Ok(results)
    } else {
        // Single roll
        let parsed_dice = parse_dice_expr(expr)?;
        Ok(vec![roll_dice(&parsed_dice)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dice_expr() {
        let dice = parse_dice_expr("2d6+1").unwrap();
        assert_eq!(dice.count, 2);
        assert_eq!(dice.sides, 6);
        assert_eq!(dice.modifier, 1);
    }

    #[test]
    fn test_parse_dice_expr_without_modifier() {
        let dice = parse_dice_expr("d20").unwrap();
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
        let results = roll_multiple_dice("+3 d4", 50).unwrap();
        assert_eq!(results.len(), 3);
    }
}