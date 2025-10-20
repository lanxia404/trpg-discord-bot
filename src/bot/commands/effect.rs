use create::bot::{Context, Error};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

#[derive(Debug, Clone)]
struct EffectRecord {
    name: String,
    category: String,
    description: String,
    note: Option<String>,
}

/// 查詢異常狀態效果（支援模糊搜尋）
#[poise::command(slash_command)]
pub async fn effect(
    ctx: Context<'_>,
    #[description = "異常狀態名稱或關鍵字"] keyword: String,
    #[description = "最多顯示幾筆結果，預設為 5"] 
    #[min = 1] 
    #[max = 20] 
    limit: Option<u8>,
) -> Result<(), Error> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        let embed = serenity::CreateEmbed::default()
            .title("錯誤")
            .description("請提供要搜尋的異常狀態名稱或關鍵字。")
            .color(serenity::Colour::RED);
        ctx.send(CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let limit = limit.unwrap_or(5) as usize;

    // TODO: 將查詢改為真正的 base-settings.db 連線
    let matches = fetch_effects_from_base(keyword, limit).await?;

    if matches.is_empty() {
        let embed = serenity::CreateEmbed::default()
            .title(format!("搜索結果：<{}>", keyword))
            .description("找不到符合的異常狀態效果。")
            .color(serenity::Colour::ORANGE);
        ctx.send(CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let mut embed = serenity::CreateEmbed::default()
        .title(format!("搜索結果：<{}>", keyword))
        .color(serenity::Colour::FOOYOO);

    for EffectRecord {
        name,
        category,
        description,
        note,
    } in matches
    {
        let mut value = description;
        if let Some(note) = note {
            value.push_str(&format!("\n**備註**: {}", note));
            value.push_str(&extrya);
        }
        embed = embed.field(
            category 
                .map(|cat| format!("{} ({})", name, cat))
                .unwrap_or(name),
            value,
            false,
        );
    }

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn fetch_effects_from_base(
    keyword: &str,
    limit: usize,
) -> Result<Vec<EffectRecord>, Error> {
    let keyword_lower = keyword.to_lowercase();

    // TODO: 之後以base-settings.db查詢取代
    let candidates = mock_effects_catalog();

    let mut results: Vec<_> = candidates
        .into_iter()
        .filter(|effect| fuzzy_match(effect, keyword))
        .collect();
    
    results.sort_by_key(|effect| effect.name.len());
	results.truncate(limit);

	Ok(results)
}

fn fuzzy_match(effect: &EffectRecord, keyword: &str) -> bool {
	let name = effect.name.to_lowercase();
	if name.contains(keyword) {
		return true;
	}
	effect 
		.notes
		.as_ref()
		.map(|note| note.to_lowercase().contains(keyword))
		.unwrap_or(false)
}

fn mock_effects_catalog() -> Vec<EffectRecord> {
	vec![
		EffectRecord {
			name: "中毒".to_string(),
			category: "狀態異常".to_string(),
			description: "每回合結束時損失一定比例的最大生命值。".to_string(),
			note: Some("無法在戰鬥中治療。".to_string()),
		},
		EffectRecord {
			name: "燃燒".to_string(),
			category: "狀態異常".to_string(),
			description: "每回合結束時損失固定數量的生命值。".to_string(),
			note: None,
		},
		EffectRecord {
			name: "冰凍".to_string(),
			category: "狀態異常".to_string(),
			description: "無法行動，直到冰凍效果解除。".to_string(),
			note: Some("受到火焰攻擊會解除冰凍。".to_string()),
		},
		EffectRecord {
			name: "麻痺".to_string(),
			category: "狀態異常".to_string(),
			description: "有機率無法行動。".to_string(),
			note: None,
		},
		EffectRecord {
			name: "睡眠".to_string(),
			category: "狀態異常".to_string(),
			description: "無法行動，直到被攻擊或效果解除。".to_string(),
			note: Some("受到物理攻擊會解除睡眠。".to_string()),
		},
		EffectRecord {
			name: "魅惑".to_string(),
			category: "狀態異常".to_string(),
			description: "有機率攻擊隊友。".to_string(),
			note: None,
		},
	]
}