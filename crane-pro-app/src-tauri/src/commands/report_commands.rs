//! Report generation command handlers
//! 
//! This module contains all Tauri command handlers for report generation
//! operations including inspection reports, compliance reports, and report management.

use crate::api::{ApiResponse, ReportFormat, DateRange, ReportResult, ReportTemplate};
use crate::commands::AppState;
use crate::middleware::{Permissions, auth::AuthHelper};
use crate::{require_auth, require_resource_access, time_command, command_handler};
use tauri::State;
use log::{info, debug, error};
use chrono::Utc;
use std::path::Path;
use std::fs;

/// Generate inspection report
#[tauri::command]
pub async fn generate_inspection_report_command(
    state: State<'_, AppState>,
    token: Option<String>,
    inspection_id: i64,
    format: ReportFormat,
) -> Result<ApiResponse<ReportResult>, String> {
    let result = time_command!("generate_inspection_report", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "report", "generate");

        // Get inspection data
        let inspection = state.services.inspections.get_inspection_by_id(inspection_id)
            .map_err(|e| format!("Failed to get inspection: {}", e))?;

        // Get asset data
        let asset = state.services.assets.get_asset_by_id(inspection.asset_id)
            .map_err(|e| format!("Failed to get asset: {}", e))?;

        // Get inspection items
        let inspection_items = state.services.inspections.get_inspection_items(inspection_id)
            .map_err(|e| format!("Failed to get inspection items: {}", e))?;

        // Get media files
        let media_files = state.services.media.get_media_files_by_inspection(inspection_id)
            .map_err(|e| format!("Failed to get media files: {}", e))?;

        // Generate report ID
        let report_id = format!("inspection_{}_{}", 
                               inspection_id, 
                               Utc::now().format("%Y%m%d_%H%M%S"));

        // Create reports directory
        let reports_dir = "./data/reports";
        fs::create_dir_all(reports_dir)
            .map_err(|e| format!("Failed to create reports directory: {}", e))?;

        // Generate report based on format
        let file_extension = match format {
            ReportFormat::Pdf => "pdf",
            ReportFormat::Html => "html",
            ReportFormat::Json => "json",
            ReportFormat::Csv => "csv",
        };

        let file_name = format!("{}.{}", report_id, file_extension);
        let file_path = format!("{}/{}", reports_dir, file_name);

        // Generate report content based on format
        match format {
            ReportFormat::Json => {
                let report_data = serde_json::json!({
                    "report_id": report_id,
                    "report_type": "inspection",
                    "generated_at": Utc::now(),
                    "inspection": {
                        "id": inspection.id,
                        "asset_id": inspection.asset_id,
                        "asset_name": asset.asset_name,
                        "asset_number": asset.asset_number,
                        "inspection_type": inspection.inspection_type,
                        "compliance_standard": inspection.compliance_standard,
                        "scheduled_date": inspection.scheduled_date,
                        "actual_date": inspection.actual_date,
                        "status": inspection.status,
                        "overall_condition": inspection.overall_condition,
                        "notes": inspection.notes
                    },
                    "items": inspection_items,
                    "media_files": media_files.iter().map(|f| serde_json::json!({
                        "id": f.id,
                        "file_name": f.file_name,
                        "file_type": f.file_type,
                        "description": f.description
                    })).collect::<Vec<_>>(),
                    "summary": {
                        "total_items": inspection_items.len(),
                        "compliant_items": inspection_items.iter().filter(|i| i.is_compliant == Some(true)).count(),
                        "non_compliant_items": inspection_items.iter().filter(|i| i.is_compliant == Some(false)).count(),
                        "critical_findings": inspection_items.iter().filter(|i| matches!(i.severity, Some(crate::models::Severity::Critical))).count(),
                        "media_count": media_files.len()
                    }
                });

                fs::write(&file_path, serde_json::to_string_pretty(&report_data).unwrap())
                    .map_err(|e| format!("Failed to write JSON report: {}", e))?;
            },
            ReportFormat::Html => {
                let html_content = generate_html_inspection_report(&inspection, &asset, &inspection_items, &media_files);
                fs::write(&file_path, html_content)
                    .map_err(|e| format!("Failed to write HTML report: {}", e))?;
            },
            ReportFormat::Csv => {
                let csv_content = generate_csv_inspection_report(&inspection, &asset, &inspection_items);
                fs::write(&file_path, csv_content)
                    .map_err(|e| format!("Failed to write CSV report: {}", e))?;
            },
            ReportFormat::Pdf => {
                // In a real implementation, this would use a PDF generation library
                let pdf_placeholder = b"PDF report generation not implemented yet";
                fs::write(&file_path, pdf_placeholder)
                    .map_err(|e| format!("Failed to write PDF report: {}", e))?;
            }
        }

        let report_result = ReportResult {
            report_id: report_id.clone(),
            format,
            file_path: Some(file_path.clone()),
            file_url: Some(format!("/api/reports/{}/download", report_id)),
            generated_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)), // Reports expire in 30 days
        };

        info!("Inspection report generated: {} for inspection {} by user {}", 
              report_id, inspection_id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(report_result)
    });

    Ok(command_handler!("generate_inspection_report", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Generate compliance report
#[tauri::command]
pub async fn generate_compliance_report_command(
    state: State<'_, AppState>,
    token: Option<String>,
    asset_id: i64,
    date_range: DateRange,
    format: ReportFormat,
) -> Result<ApiResponse<ReportResult>, String> {
    let result = time_command!("generate_compliance_report", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "report", "generate");

        // Get asset data
        let asset = state.services.assets.get_asset_by_id(asset_id)
            .map_err(|e| format!("Failed to get asset: {}", e))?;

        // Get compliance status report
        let compliance_report = state.services.reports.generate_compliance_status_report(Some(asset.location_id))
            .map_err(|e| format!("Failed to generate compliance status: {}", e))?;

        // Generate report ID
        let report_id = format!("compliance_{}_{}", 
                               asset_id, 
                               Utc::now().format("%Y%m%d_%H%M%S"));

        // Create reports directory
        let reports_dir = "./data/reports";
        fs::create_dir_all(reports_dir)
            .map_err(|e| format!("Failed to create reports directory: {}", e))?;

        // Generate report based on format
        let file_extension = match format {
            ReportFormat::Pdf => "pdf",
            ReportFormat::Html => "html",
            ReportFormat::Json => "json",
            ReportFormat::Csv => "csv",
        };

        let file_name = format!("{}.{}", report_id, file_extension);
        let file_path = format!("{}/{}", reports_dir, file_name);

        // Generate report content
        match format {
            ReportFormat::Json => {
                let report_data = serde_json::json!({
                    "report_id": report_id,
                    "report_type": "compliance",
                    "generated_at": Utc::now(),
                    "date_range": {
                        "start_date": date_range.start_date,
                        "end_date": date_range.end_date
                    },
                    "asset": {
                        "id": asset.id,
                        "name": asset.asset_name,
                        "asset_number": asset.asset_number,
                        "type": asset.asset_type,
                        "location_id": asset.location_id
                    },
                    "compliance_status": compliance_report
                });

                fs::write(&file_path, serde_json::to_string_pretty(&report_data).unwrap())
                    .map_err(|e| format!("Failed to write JSON compliance report: {}", e))?;
            },
            ReportFormat::Html => {
                let html_content = generate_html_compliance_report(&asset, &compliance_report, &date_range);
                fs::write(&file_path, html_content)
                    .map_err(|e| format!("Failed to write HTML compliance report: {}", e))?;
            },
            ReportFormat::Csv => {
                let csv_content = generate_csv_compliance_report(&asset, &compliance_report);
                fs::write(&file_path, csv_content)
                    .map_err(|e| format!("Failed to write CSV compliance report: {}", e))?;
            },
            ReportFormat::Pdf => {
                // PDF generation placeholder
                let pdf_placeholder = b"PDF compliance report generation not implemented yet";
                fs::write(&file_path, pdf_placeholder)
                    .map_err(|e| format!("Failed to write PDF compliance report: {}", e))?;
            }
        }

        let report_result = ReportResult {
            report_id: report_id.clone(),
            format,
            file_path: Some(file_path.clone()),
            file_url: Some(format!("/api/reports/{}/download", report_id)),
            generated_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
        };

        info!("Compliance report generated: {} for asset {} by user {}", 
              report_id, asset_id,
              context.current_user().map(|u| u.user_id).unwrap_or(0));

        Ok(report_result)
    });

    Ok(command_handler!("generate_compliance_report", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// Get report by ID
#[tauri::command]
pub async fn get_report_command(
    state: State<'_, AppState>,
    token: Option<String>,
    report_id: String,
) -> Result<ApiResponse<ReportResult>, String> {
    let result = time_command!("get_report", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "report", "read");

        // Check if report file exists
        let reports_dir = "./data/reports";
        let possible_extensions = ["pdf", "html", "json", "csv"];
        
        let mut found_file = None;
        let mut found_format = None;
        
        for ext in &possible_extensions {
            let file_path = format!("{}/{}.{}", reports_dir, report_id, ext);
            if Path::new(&file_path).exists() {
                found_file = Some(file_path);
                found_format = Some(match *ext {
                    "pdf" => ReportFormat::Pdf,
                    "html" => ReportFormat::Html,
                    "json" => ReportFormat::Json,
                    "csv" => ReportFormat::Csv,
                    _ => ReportFormat::Json,
                });
                break;
            }
        }

        let (file_path, format) = match (found_file, found_format) {
            (Some(path), Some(fmt)) => (path, fmt),
            _ => return Err(format!("Report not found: {}", report_id)),
        };

        // Get file metadata
        let metadata = fs::metadata(&file_path)
            .map_err(|e| format!("Failed to get report metadata: {}", e))?;

        let report_result = ReportResult {
            report_id: report_id.clone(),
            format,
            file_path: Some(file_path),
            file_url: Some(format!("/api/reports/{}/download", report_id)),
            generated_at: metadata.created()
                .map(|t| chrono::DateTime::from(t))
                .unwrap_or_else(|_| Utc::now()),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
        };

        debug!("Report retrieved: {}", report_id);
        Ok(report_result)
    });

    Ok(command_handler!("get_report", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

/// List available report templates
#[tauri::command]
pub async fn list_available_reports_command(
    state: State<'_, AppState>,
    token: Option<String>,
) -> Result<ApiResponse<Vec<ReportTemplate>>, String> {
    let result = time_command!("list_available_reports", {
        // Authenticate and authorize
        let context = AuthHelper::validate_request(&state.auth_manager, token)
            .map_err(|e| format!("Authentication failed: {}", e))?;
        
        require_resource_access!(context, "report", "read");

        // Define available report templates
        let templates = vec![
            ReportTemplate {
                id: "inspection_report".to_string(),
                name: "Inspection Report".to_string(),
                description: "Detailed report for a single inspection including items, findings, and media".to_string(),
                supported_formats: vec![
                    ReportFormat::Pdf,
                    ReportFormat::Html,
                    ReportFormat::Json,
                    ReportFormat::Csv,
                ],
                parameters: vec![
                    crate::api::ReportParameter {
                        name: "inspection_id".to_string(),
                        parameter_type: "integer".to_string(),
                        required: true,
                        description: "ID of the inspection to generate report for".to_string(),
                        default_value: None,
                    },
                    crate::api::ReportParameter {
                        name: "format".to_string(),
                        parameter_type: "string".to_string(),
                        required: true,
                        description: "Report format (pdf, html, json, csv)".to_string(),
                        default_value: Some("pdf".to_string()),
                    },
                ],
            },
            ReportTemplate {
                id: "compliance_report".to_string(),
                name: "Compliance Report".to_string(),
                description: "Compliance status report for an asset over a date range".to_string(),
                supported_formats: vec![
                    ReportFormat::Pdf,
                    ReportFormat::Html,
                    ReportFormat::Json,
                    ReportFormat::Csv,
                ],
                parameters: vec![
                    crate::api::ReportParameter {
                        name: "asset_id".to_string(),
                        parameter_type: "integer".to_string(),
                        required: true,
                        description: "ID of the asset to generate compliance report for".to_string(),
                        default_value: None,
                    },
                    crate::api::ReportParameter {
                        name: "date_range".to_string(),
                        parameter_type: "object".to_string(),
                        required: true,
                        description: "Date range for the compliance report".to_string(),
                        default_value: None,
                    },
                    crate::api::ReportParameter {
                        name: "format".to_string(),
                        parameter_type: "string".to_string(),
                        required: true,
                        description: "Report format (pdf, html, json, csv)".to_string(),
                        default_value: Some("pdf".to_string()),
                    },
                ],
            },
        ];

        debug!("Listed {} available report templates", templates.len());
        Ok(templates)
    });

    Ok(command_handler!("list_available_reports", 
                       result.as_ref().ok().and_then(|_| None), 
                       { result }))
}

// Helper functions for report generation

fn generate_html_inspection_report(
    inspection: &crate::models::Inspection,
    asset: &crate::models::Asset,
    items: &[crate::models::InspectionItem],
    media_files: &[crate::models::MediaFile],
) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Inspection Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1, h2 {{ color: #333; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .summary {{ background-color: #f9f9f9; padding: 15px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>Inspection Report</h1>
    <div class="summary">
        <h2>Asset Information</h2>
        <p><strong>Asset Name:</strong> {}</p>
        <p><strong>Asset Number:</strong> {}</p>
        <p><strong>Asset Type:</strong> {}</p>
        
        <h2>Inspection Details</h2>
        <p><strong>Inspection ID:</strong> {}</p>
        <p><strong>Inspection Type:</strong> {:?}</p>
        <p><strong>Status:</strong> {:?}</p>
        <p><strong>Scheduled Date:</strong> {:?}</p>
        <p><strong>Actual Date:</strong> {:?}</p>
        <p><strong>Overall Condition:</strong> {:?}</p>
    </div>
    
    <h2>Inspection Items</h2>
    <table>
        <tr>
            <th>Item Name</th>
            <th>Category</th>
            <th>Condition</th>
            <th>Finding</th>
            <th>Severity</th>
            <th>Compliant</th>
        </tr>
        {}
    </table>
    
    <h2>Media Files</h2>
    <p>Total media files: {}</p>
    
    <p><em>Generated on: {}</em></p>
</body>
</html>
"#,
        asset.asset_name,
        asset.asset_name,
        asset.asset_number,
        asset.asset_type,
        inspection.id,
        inspection.inspection_type,
        inspection.status,
        inspection.scheduled_date,
        inspection.actual_date,
        inspection.overall_condition,
        items.iter().map(|item| format!(
            "<tr><td>{}</td><td>{}</td><td>{:?}</td><td>{}</td><td>{:?}</td><td>{}</td></tr>",
            item.item_name,
            item.item_category,
            item.condition,
            item.finding.as_deref().unwrap_or("N/A"),
            item.severity,
            item.is_compliant.map(|c| if c { "Yes" } else { "No" }).unwrap_or("N/A")
        )).collect::<Vec<_>>().join(""),
        media_files.len(),
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
}

fn generate_csv_inspection_report(
    inspection: &crate::models::Inspection,
    asset: &crate::models::Asset,
    items: &[crate::models::InspectionItem],
) -> String {
    let mut csv = String::new();
    csv.push_str("Asset Name,Asset Number,Inspection ID,Item Name,Category,Condition,Finding,Severity,Compliant\n");
    
    for item in items {
        csv.push_str(&format!(
            "{},{},{},{},{},{:?},{},{:?},{}\n",
            asset.asset_name,
            asset.asset_number,
            inspection.id,
            item.item_name,
            item.item_category,
            item.condition,
            item.finding.as_deref().unwrap_or(""),
            item.severity,
            item.is_compliant.map(|c| if c { "Yes" } else { "No" }).unwrap_or("N/A")
        ));
    }
    
    csv
}

fn generate_html_compliance_report(
    asset: &crate::models::Asset,
    compliance_report: &crate::services::ComplianceStatusReport,
    date_range: &DateRange,
) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Compliance Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1, h2 {{ color: #333; }}
        .summary {{ background-color: #f9f9f9; padding: 15px; border-radius: 5px; }}
        .metric {{ margin: 10px 0; }}
    </style>
</head>
<body>
    <h1>Compliance Report</h1>
    <div class="summary">
        <h2>Asset Information</h2>
        <p><strong>Asset Name:</strong> {}</p>
        <p><strong>Asset Number:</strong> {}</p>
        
        <h2>Report Period</h2>
        <p><strong>From:</strong> {}</p>
        <p><strong>To:</strong> {}</p>
        
        <h2>Compliance Summary</h2>
        <div class="metric"><strong>Total Assets:</strong> {}</div>
        <div class="metric"><strong>Compliant Assets:</strong> {}</div>
        <div class="metric"><strong>Non-Compliant Assets:</strong> {}</div>
        <div class="metric"><strong>Compliance Percentage:</strong> {:.1}%</div>
        <div class="metric"><strong>Critical Findings:</strong> {}</div>
        <div class="metric"><strong>Overdue Inspections:</strong> {}</div>
    </div>
    
    <p><em>Generated on: {}</em></p>
</body>
</html>
"#,
        asset.asset_name,
        asset.asset_name,
        asset.asset_number,
        date_range.start_date.format("%Y-%m-%d"),
        date_range.end_date.format("%Y-%m-%d"),
        compliance_report.total_assets,
        compliance_report.compliant_assets,
        compliance_report.non_compliant_assets,
        compliance_report.compliance_percentage,
        compliance_report.critical_findings,
        compliance_report.overdue_inspections,
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
}

fn generate_csv_compliance_report(
    asset: &crate::models::Asset,
    compliance_report: &crate::services::ComplianceStatusReport,
) -> String {
    format!(
        "Asset Name,Asset Number,Total Assets,Compliant Assets,Non-Compliant Assets,Compliance Percentage,Critical Findings,Overdue Inspections\n{},{},{},{},{},{:.1},{},{}\n",
        asset.asset_name,
        asset.asset_number,
        compliance_report.total_assets,
        compliance_report.compliant_assets,
        compliance_report.non_compliant_assets,
        compliance_report.compliance_percentage,
        compliance_report.critical_findings,
        compliance_report.overdue_inspections
    )
}