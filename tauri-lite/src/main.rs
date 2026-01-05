//! Task NeXus Lite - 原生高性能 CPU 调度工具

mod core;

use eframe::egui;
use std::collections::HashSet;
use sysinfo::System;
use core::{governor, thread, topology, PriorityLevel, CpuTopology, AppConfig, set_auto_start};

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([880.0, 700.0])
            .with_min_inner_size([650.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Task NeXus Lite",
        options,
        Box::new(|cc| {
            // 配置中文字体支持
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "msyh".to_owned(),
                egui::FontData::from_static(include_bytes!("C:\\Windows\\Fonts\\msyh.ttc")),
            );
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "msyh".to_owned());
            fonts.families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("msyh".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(147, 51, 234);
            cc.egui_ctx.set_style(style);
            
            Box::new(TNLiteApp::new())
        }),
    )
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../icon.ico");
    if let Ok(icon_dir) = ico::IconDir::read(std::io::Cursor::new(icon_bytes)) {
        if let Some(entry) = icon_dir.entries().iter().max_by_key(|e| e.width()) {
            if let Ok(image) = entry.decode() {
                return egui::IconData {
                    rgba: image.rgba_data().to_vec(),
                    width: image.width(),
                    height: image.height(),
                };
            }
        }
    }
    egui::IconData::default()
}

struct TNLiteApp {
    sys: System,
    selected_cores: HashSet<usize>,
    processes: Vec<LiteProcess>,
    search_term: String,
    last_refresh: std::time::Instant,
    topology: Option<CpuTopology>,
    config: AppConfig,
    status_msg: String,
}

struct LiteProcess {
    pid: u32,
    name: String,
    cpu: f32,
    mem: u64,
}

impl TNLiteApp {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let topology = topology::get_cpu_topology().ok();
        let core_count = sys.cpus().len();
        let config = AppConfig::load();
        
        Self {
            sys,
            selected_cores: (0..core_count).collect(),
            processes: Vec::new(),
            search_term: String::new(),
            last_refresh: std::time::Instant::now(),
            topology,
            config,
            status_msg: String::new(),
        }
    }

    fn refresh_data(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
        
        let mut new_procs = Vec::new();
        let query = self.search_term.to_lowercase();
        
        for (pid, process) in self.sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            if name.is_empty() { continue; }
            if !query.is_empty() && !name.to_lowercase().contains(&query) { continue; }
            new_procs.push(LiteProcess {
                pid: pid.as_u32(),
                name,
                cpu: process.cpu_usage() / self.sys.cpus().len() as f32,
                mem: process.memory() / 1024 / 1024,
            });
        }
        new_procs.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
        self.processes = new_procs;
    }
}

impl eframe::App for TNLiteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_refresh.elapsed() >= std::time::Duration::from_millis(1000) {
            self.refresh_data();
            self.last_refresh = std::time::Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题栏
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Task NeXus Lite").color(egui::Color32::from_rgb(180, 100, 255)));
                if let Some(top) = &self.topology {
                    ui.label(format!("| {} | {}核/{}线程", top.model, top.physical_cores, top.logical_cores));
                }
            });

            ui.add_space(6.0);

            // 核心选择网格 (无热力图)
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong("核心选择");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("全选").clicked() { 
                            let count = self.topology.as_ref().map(|t| t.logical_cores as usize).unwrap_or(16);
                            self.selected_cores = (0..count).collect(); 
                        }
                        if ui.button("清空").clicked() { self.selected_cores.clear(); }
                    });
                });
                ui.add_space(4.0);
                let core_count = self.topology.as_ref().map(|t| t.logical_cores as usize).unwrap_or(16);
                let cols = (ui.available_width() / 50.0).floor() as usize;
                let cols = if cols == 0 { 1 } else { cols };
                egui::Grid::new("core_grid").spacing([4.0, 4.0]).show(ui, |ui| {
                    for i in 0..core_count {
                        let is_selected = self.selected_cores.contains(&i);
                        let color = if is_selected { egui::Color32::from_rgb(147, 51, 234) } else { egui::Color32::from_gray(60) };
                        let button = egui::Button::new(egui::RichText::new(format!("{}", i)).color(egui::Color32::WHITE))
                            .fill(color).min_size(egui::vec2(42.0, 42.0)).rounding(4.0);
                        if ui.add(button).clicked() {
                            if is_selected { self.selected_cores.remove(&i); } else { self.selected_cores.insert(i); }
                        }
                        if (i + 1) % cols == 0 { ui.end_row(); }
                    }
                });
            });

            ui.add_space(6.0);

            // 进程列表
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong("进程列表");
                    ui.add(egui::TextEdit::singleline(&mut self.search_term).hint_text("搜索...").desired_width(200.0));
                });
                ui.add_space(4.0);
                
                // 表头
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("进程名").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("操作").strong());
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("CPU").strong());
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("内存").strong());
                    });
                });
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(350.0).id_source("proc_scroll").show(ui, |ui| {
                    for proc in &self.processes {
                        let proc_pid = proc.pid;
                        let proc_name = proc.name.clone();
                        let selected_cores = self.selected_cores.clone();
                        let core_count = self.topology.as_ref().map(|t| t.logical_cores as usize).unwrap_or(16);

                        ui.push_id(proc_pid, |ui| {
                            let response = ui.horizontal(|ui| {
                                // 进程名 (明确显示)
                                let is_game = proc_name.to_lowercase().contains("cs2") || proc_name.to_lowercase().contains("apex");
                                let name_color = if is_game { egui::Color32::from_rgb(100, 255, 100) } else { egui::Color32::from_rgb(220, 220, 220) };
                                ui.label(egui::RichText::new(&proc_name).color(name_color).strong());
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("应用").clicked() {
                                        let mut mask: u64 = 0;
                                        for &c in &selected_cores { mask |= 1 << c; }
                                        let mask_str = format!("{:X}", mask);
                                        tokio::spawn(async move {
                                            let _ = governor::set_process_affinity(proc_pid, mask_str).await;
                                        });
                                    }
                                    ui.add_space(10.0);
                                    ui.label(format!("{:.1}%", proc.cpu));
                                    ui.add_space(10.0);
                                    ui.label(format!("{} MB", proc.mem));
                                });
                            }).response;

                            response.context_menu(|ui| {
                                ui.label(egui::RichText::new(format!("{}", proc_name)).strong());
                                ui.separator();
                                ui.menu_button("优先级", |ui| {
                                    for (label, p_str) in [("实时", "RealTime"), ("高", "High"), ("高于正常", "AboveNormal"), ("正常", "Normal"), ("低于正常", "BelowNormal"), ("空闲", "Idle")] {
                                        if ui.button(label).clicked() { 
                                            if let Some(level) = PriorityLevel::from_str(p_str) {
                                                tokio::spawn(async move {
                                                    let _ = governor::set_priority(proc_pid, level).await;
                                                });
                                            }
                                            ui.close_menu();
                                        }
                                    }
                                });
                                ui.menu_button("线程绑定", |ui| {
                                    for i in 0..core_count {
                                        if ui.button(format!("核心 {}", i)).clicked() {
                                            tokio::spawn(async move {
                                                let _ = thread::smart_bind_thread(proc_pid, i as u32).await;
                                            });
                                            ui.close_menu();
                                        }
                                    }
                                });
                                ui.separator();
                                if ui.button(egui::RichText::new("结束进程").color(egui::Color32::LIGHT_RED)).clicked() {
                                    tokio::spawn(async move {
                                        let _ = governor::kill_process(proc_pid).await;
                                    });
                                    ui.close_menu();
                                }
                            });
                        });
                        ui.separator();
                    }
                });
            });

            // 底部控制栏
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    // 开机自启动
                    let mut auto_start = self.config.auto_start;
                    if ui.checkbox(&mut auto_start, "开机自启动").changed() {
                        self.config.auto_start = auto_start;
                        if set_auto_start(auto_start).is_ok() {
                            let _ = self.config.save();
                            self.status_msg = if auto_start { "已启用开机自启动".to_string() } else { "已禁用开机自启动".to_string() };
                        }
                    }
                    
                    ui.separator();
                    
                    if ui.button("保存配置").clicked() {
                        if self.config.save().is_ok() {
                            self.status_msg = "配置已保存".to_string();
                        }
                    }
                    
                    if ui.button("导出配置").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .set_file_name("tn_lite_config.json")
                            .save_file() 
                        {
                            if self.config.export_to(&path).is_ok() {
                                self.status_msg = format!("已导出到: {}", path.display());
                            }
                        }
                    }
                    
                    if ui.button("导入配置").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file() 
                        {
                            if let Ok(config) = AppConfig::import_from(&path) {
                                self.config = config;
                                let _ = self.config.save();
                                self.status_msg = "配置已导入".to_string();
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    if ui.button("清理内存").clicked() {
                        tokio::spawn(async move {
                            let _ = governor::clear_system_memory().await;
                        });
                        self.status_msg = "内存清理已触发".to_string();
                    }
                });
                
                if !self.status_msg.is_empty() {
                    ui.label(egui::RichText::new(&self.status_msg).weak());
                }
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(200));
    }
}
