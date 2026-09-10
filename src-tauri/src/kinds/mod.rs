pub mod registry;

use serde_json::json;

use crate::error::{AppError, Result};
use crate::kinds::registry::KindDescriptor;
use crate::models::EntryWrite;

pub fn default_payload() -> String {
    json!({ "v": 1 }).to_string()
}

pub fn normalize_write(input: &mut EntryWrite) {
    if let Some(desc) = registry::get(&input.kind_id) {
        if !desc.counter_account_required {
            input.counter_account_id = None;
        }
        if !desc.counter_amount_required {
            input.counter_amount_minor = None;
            input.fee_category_id = None;
        }
        if desc.implemented {
            input.kind_payload = Some(default_payload());
        }
    }
}

pub fn validate_kind_fields(desc: &KindDescriptor, input: &EntryWrite) -> Result<()> {
    if input.amount_minor <= 0 {
        return Err(AppError::new("error.amountInvalid"));
    }
    if desc.category_required && input.category_id.trim().is_empty() {
        return Err(AppError::new("error.categoryRequired"));
    }
    if desc.counter_amount_required {
        validate_transfer(input)
    } else if desc.counter_account_required {
        validate_counter_account_only(input)
    } else {
        validate_single_account(input)
    }
}

fn validate_single_account(input: &EntryWrite) -> Result<()> {
    if input.counter_account_id.is_some()
        || input.counter_amount_minor.is_some()
        || input.fee_category_id.is_some()
    {
        return Err(AppError::new("error.counterpartyMustBeEmpty"));
    }
    Ok(())
}

fn validate_counter_account_only(input: &EntryWrite) -> Result<()> {
    let dest = input
        .counter_account_id
        .as_ref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("error.counterAccountRequired"))?;
    if dest == &input.account_id {
        return Err(AppError::new("error.accountsMustDiffer"));
    }
    if input.counter_amount_minor.is_some() || input.fee_category_id.is_some() {
        return Err(AppError::new("error.counterpartyMustBeEmpty"));
    }
    Ok(())
}

fn validate_transfer(input: &EntryWrite) -> Result<()> {
    let dest = input
        .counter_account_id
        .as_ref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::new("error.counterpartyRequired"))?;
    if dest == &input.account_id {
        return Err(AppError::new("error.transferSameAccount"));
    }
    let counter = input
        .counter_amount_minor
        .ok_or_else(|| AppError::new("error.counterpartyRequired"))?;
    if counter <= 0 {
        return Err(AppError::new("error.counterAmountInvalid"));
    }
    if counter > input.amount_minor {
        return Err(AppError::new("error.transferDestExceedsSource"));
    }
    let fee = input.amount_minor - counter;
    if fee == 0 {
        if input.fee_category_id.is_some() {
            return Err(AppError::new("error.feeCategoryMustBeNull"));
        }
    } else if input
        .fee_category_id
        .as_ref()
        .map(|s| s.is_empty())
        .unwrap_or(true)
    {
        return Err(AppError::new("error.feeCategoryRequired"));
    }
    Ok(())
}

pub fn require_implemented(kind_id: &str) -> Result<&'static KindDescriptor> {
    let desc = registry::get(kind_id).ok_or_else(|| AppError::new("error.kindUnknown"))?;
    if !desc.implemented {
        return Err(AppError::new("error.kindNotImplemented"));
    }
    Ok(desc)
}
