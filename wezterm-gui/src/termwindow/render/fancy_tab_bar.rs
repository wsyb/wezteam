use crate::customglyph::*;
use crate::tabbar::{parse_status_text, TabBarItem, TabEntry};
use crate::termwindow::box_model::*;
use crate::termwindow::render::corners::*;
use crate::termwindow::tab_extra_info::get_extra_info;

use crate::termwindow::render::window_buttons::window_button_element;
use crate::termwindow::{UIItem, UIItemType};
use crate::utilsprites::RenderMetrics;
use config::{Dimension, DimensionContext, TabBarColors};
use std::rc::Rc;
use termwiz::cell::CellAttributes;
use wezterm_font::LoadedFont;
use wezterm_term::color::{AnsiColor, ColorAttribute, ColorPalette};
use window::color::LinearRgba;
use window::{IntegratedTitleButtonAlignment, IntegratedTitleButtonStyle};

const X_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

const PLUS_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

impl crate::TermWindow {
    pub fn invalidate_fancy_tab_bar(&mut self) {
        self.fancy_tab_bar.take();
    }

    fn tab_bar_colors(&self) -> TabBarColors {
        self.config
            .colors
            .as_ref()
            .and_then(|c| c.tab_bar.as_ref())
            .cloned()
            .unwrap_or_else(TabBarColors::default)
    }

    fn titlebar_bg_linear(&self) -> LinearRgba {
        if self.focused.is_some() {
            self.config.window_frame.active_titlebar_bg
        } else {
            self.config.window_frame.inactive_titlebar_bg
        }
        .to_linear()
    }

    fn bar_element_colors(&self) -> ElementColors {
        ElementColors {
            border: BorderColor::default(),
            bg: self.titlebar_bg_linear().into(),
            text: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_fg
            } else {
                self.config.window_frame.inactive_titlebar_fg
            }
            .to_linear()
            .into(),
        }
    }

    pub fn build_fancy_tab_bar(&self, palette: &ColorPalette) -> anyhow::Result<ComputedElement> {
        if self.config.tab_bar_vertical {
            return self.build_vertical_fancy_tab_bar(palette);
        }

        let tab_bar_height = self.tab_bar_pixel_height()?;
        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let items = self.tab_bar.items();
        let colors = self.tab_bar_colors();

        let mut left_status = vec![];
        let mut left_eles = vec![];
        let mut right_eles = vec![];
        let bar_colors = self.bar_element_colors();

        let item_to_elem = |item: &TabEntry| -> Element {
            let element = Element::with_line(&font, &item.title, palette);

            let bg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().background() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_bg(col)),
                });
            let fg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().foreground() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_fg(col)),
                });

            let new_tab = colors.new_tab();
            let new_tab_hover = colors.new_tab_hover();
            let active_tab = colors.active_tab();

            match item.item {
                TabBarItem::RightStatus | TabBarItem::LeftStatus | TabBarItem::None => element
                    .item_type(UIItemType::TabBar(TabBarItem::None))
                    .line_height(Some(1.75))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.0),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.),
                        bottom: Dimension::Cells(0.),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(0.)))
                    .colors(bar_colors.clone()),
                TabBarItem::NewTabButton => Element::new(
                    &font,
                    ElementContent::Poly {
                        line_width: metrics.underline_height.max(2),
                        poly: SizedPoly {
                            poly: PLUS_BUTTON,
                            width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                            height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                        },
                    },
                )
                .vertical_align(VerticalAlign::Middle)
                .item_type(UIItemType::TabBar(item.item.clone()))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.25),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.)))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab.bg_color.to_linear().into(),
                    text: new_tab.fg_color.to_linear().into(),
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab_hover.bg_color.to_linear().into(),
                    text: new_tab_hover.fg_color.to_linear().into(),
                })),
                TabBarItem::Tab { active, .. } if active => element
                    .vertical_align(VerticalAlign::Bottom)
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.5),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.25),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(1.)))
                    .border_corners(Some(Corners {
                        top_left: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        bottom_left: SizedPoly::none(),
                        bottom_right: SizedPoly::none(),
                    }))
                    .colors(ElementColors {
                        border: BorderColor::new(
                            bg_color
                                .unwrap_or_else(|| active_tab.bg_color.into())
                                .to_linear(),
                        ),
                        bg: bg_color
                            .unwrap_or_else(|| active_tab.bg_color.into())
                            .to_linear()
                            .into(),
                        text: fg_color
                            .unwrap_or_else(|| active_tab.fg_color.into())
                            .to_linear()
                            .into(),
                    }),
                TabBarItem::Tab { .. } => element
                    .vertical_align(VerticalAlign::Bottom)
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.5),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.25),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(1.)))
                    .border_corners(Some(Corners {
                        top_left: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        bottom_left: SizedPoly {
                            width: Dimension::Cells(0.),
                            height: Dimension::Cells(0.33),
                            poly: &[],
                        },
                        bottom_right: SizedPoly {
                            width: Dimension::Cells(0.),
                            height: Dimension::Cells(0.33),
                            poly: &[],
                        },
                    }))
                    .colors({
                        let inactive_tab = colors.inactive_tab();
                        let bg = bg_color
                            .unwrap_or_else(|| inactive_tab.bg_color.into())
                            .to_linear();
                        let edge = colors.inactive_tab_edge().to_linear();
                        ElementColors {
                            border: BorderColor {
                                left: bg,
                                right: edge,
                                top: bg,
                                bottom: bg,
                            },
                            bg: bg.into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab.fg_color.into())
                                .to_linear()
                                .into(),
                        }
                    })
                    .hover_colors({
                        let inactive_tab_hover = colors.inactive_tab_hover();
                        Some(ElementColors {
                            border: BorderColor::new(
                                bg_color
                                    .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                    .to_linear(),
                            ),
                            bg: bg_color
                                .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                .to_linear()
                                .into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab_hover.fg_color.into())
                                .to_linear()
                                .into(),
                        })
                    }),
                TabBarItem::WindowButton(button) => window_button_element(
                    button,
                    self.window_state.contains(window::WindowState::MAXIMIZED),
                    &font,
                    &metrics,
                    &self.config,
                ),
            }
        };

        let num_tabs: f32 = items
            .iter()
            .map(|item| match item.item {
                TabBarItem::NewTabButton | TabBarItem::Tab { .. } => 1.,
                _ => 0.,
            })
            .sum();
        let max_tab_width = ((self.dimensions.pixel_width as f32 / num_tabs)
            - (1.5 * metrics.cell_size.width as f32))
            .max(0.);

        // Reserve space for the native titlebar buttons
        if self
            .config
            .window_decorations
            .contains(::window::WindowDecorations::INTEGRATED_BUTTONS)
            && self.config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            && !self.window_state.contains(window::WindowState::FULL_SCREEN)
        {
            left_status.push(
                Element::new(&font, ElementContent::Text("".to_string())).margin(BoxDimension {
                    left: Dimension::Cells(4.0), // FIXME: determine exact width of macos ... buttons
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                }),
            );
        }

        for item in items {
            match item.item {
                TabBarItem::LeftStatus => left_status.push(item_to_elem(item)),
                TabBarItem::None | TabBarItem::RightStatus => right_eles.push(item_to_elem(item)),
                TabBarItem::WindowButton(_) => {
                    if self.config.integrated_title_button_alignment
                        == IntegratedTitleButtonAlignment::Left
                    {
                        left_eles.push(item_to_elem(item))
                    } else {
                        right_eles.push(item_to_elem(item))
                    }
                }
                TabBarItem::Tab { tab_idx, active } => {
                    let mut elem = item_to_elem(item);
                    elem.max_width = Some(Dimension::Pixels(max_tab_width));
                    elem.content = match elem.content {
                        ElementContent::Text(_) => unreachable!(),
                        ElementContent::Poly { .. } => unreachable!(),
                        ElementContent::Children(mut kids) => {
                            if self.config.show_close_tab_button_in_tabs {
                                kids.push(make_x_button(&font, &metrics, &colors, tab_idx, active));
                            }
                            ElementContent::Children(kids)
                        }
                    };
                    left_eles.push(elem);
                }
                _ => left_eles.push(item_to_elem(item)),
            }
        }

        let mut children = vec![];

        if !left_status.is_empty() {
            children.push(
                Element::new(&font, ElementContent::Children(left_status))
                    .colors(bar_colors.clone()),
            );
        }

        let window_buttons_at_left = self
            .config
            .window_decorations
            .contains(window::WindowDecorations::INTEGRATED_BUTTONS)
            && (self.config.integrated_title_button_alignment
                == IntegratedTitleButtonAlignment::Left
                || self.config.integrated_title_button_style
                    == IntegratedTitleButtonStyle::MacOsNative);

        let left_padding = if window_buttons_at_left {
            if self.config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            {
                if !self.window_state.contains(window::WindowState::FULL_SCREEN) {
                    Dimension::Pixels(70.0)
                } else {
                    Dimension::Cells(0.5)
                }
            } else {
                Dimension::Pixels(0.0)
            }
        } else {
            Dimension::Cells(0.5)
        };

        children.push(
            Element::new(&font, ElementContent::Children(left_eles))
                .vertical_align(VerticalAlign::Bottom)
                .colors(bar_colors.clone())
                .padding(BoxDimension {
                    left: left_padding,
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .zindex(1),
        );
        children.push(
            Element::new(&font, ElementContent::Children(right_eles))
                .colors(bar_colors.clone())
                .float(Float::Right),
        );

        let content = ElementContent::Children(children);

        let tabs = Element::new(&font, content)
            .display(DisplayType::Block)
            .item_type(UIItemType::TabBar(TabBarItem::None))
            .min_width(Some(Dimension::Pixels(self.dimensions.pixel_width as f32)))
            .min_height(Some(Dimension::Pixels(tab_bar_height)))
            .vertical_align(VerticalAlign::Bottom)
            .colors(bar_colors);

        let border = self.get_os_border();

        let mut computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(
                    border.left.get() as f32,
                    0.,
                    self.dimensions.pixel_width as f32 - (border.left + border.right).get() as f32,
                    tab_bar_height,
                ),
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 10,
            },
            &tabs,
        )?;

        computed.translate(euclid::vec2(
            0.,
            if self.config.tab_bar_at_bottom {
                self.dimensions.pixel_height as f32
                    - (computed.bounds.height() + border.bottom.get() as f32)
            } else {
                border.top.get() as f32
            },
        ));

        Ok(computed)
    }

    /// Build a vertical tab bar rendered on the left side of the window.
    /// Each tab is a Block element so they stack vertically.
    fn build_vertical_fancy_tab_bar(
        &self,
        palette: &ColorPalette,
    ) -> anyhow::Result<ComputedElement> {
        let tab_bar_width = self.tab_bar_pixel_width();
        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let items = self.tab_bar.items();
        let colors = self.tab_bar_colors();
        let bar_colors = self.bar_element_colors();
        
        log::info!("build_vertical_fancy_tab_bar: tab_bar_width={}, window_width={}, override={:?}", 
            tab_bar_width, 
            self.dimensions.pixel_width,
            self.vertical_tab_bar_width_override);

        let active_tab_colors = colors.active_tab();
        let new_tab_colors = colors.new_tab();
        let new_tab_hover_colors = colors.new_tab_hover();

        // Get tab information for extra info
        let tabs_info = self.get_tab_information();
        let show_extra = self.config.tab_bar_vertical_extra_info;
        
        log::info!("build_vertical_fancy_tab_bar: show_extra={}, tabs_info.len()={}", show_extra, tabs_info.len());

        let mut tab_children = vec![];

        for (idx, item) in items.iter().enumerate() {
            let element = Element::with_line(&font, &item.title, palette);

            let bg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().background() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_bg(col)),
                });
            let fg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().foreground() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_fg(col)),
                });

            match item.item {
                TabBarItem::Tab { tab_idx, active } => {
                    let mut elem = element
                        .item_type(UIItemType::TabBar(item.item.clone()))
                        .display(DisplayType::Block)
                        .max_width(Some(Dimension::Pixels(tab_bar_width)))
                        .padding(BoxDimension {
                            left: Dimension::Cells(0.5),
                            right: Dimension::Cells(0.5),
                            top: Dimension::Cells(0.2),
                            bottom: Dimension::Cells(0.2),
                        })
                        .margin(BoxDimension {
                            left: Dimension::Pixels(0.),
                            right: Dimension::Pixels(0.),
                            top: Dimension::Pixels(1.),
                            bottom: Dimension::Pixels(0.),
                        })
                        .border(BoxDimension::new(Dimension::Pixels(1.)));

                    if active {
                        // Active tab: highlighted with a left border accent
                        elem = elem.border_corners(Some(Corners {
                            top_left: SizedPoly {
                                width: Dimension::Cells(0.3),
                                height: Dimension::Cells(0.3),
                                poly: TOP_LEFT_ROUNDED_CORNER,
                            },
                            top_right: SizedPoly::none(),
                            bottom_left: SizedPoly {
                                width: Dimension::Cells(0.3),
                                height: Dimension::Cells(0.3),
                                poly: BOTTOM_LEFT_ROUNDED_CORNER,
                            },
                            bottom_right: SizedPoly::none(),
                        }))
                        .colors(ElementColors {
                            border: BorderColor::new(
                                bg_color
                                    .unwrap_or_else(|| active_tab_colors.bg_color.into())
                                    .to_linear(),
                            ),
                            bg: bg_color
                                .unwrap_or_else(|| active_tab_colors.bg_color.into())
                                .to_linear()
                                .into(),
                            text: fg_color
                                .unwrap_or_else(|| active_tab_colors.fg_color.into())
                                .to_linear()
                                .into(),
                        });
                    } else {
                        let inactive_tab = colors.inactive_tab();
                        let inactive_tab_hover = colors.inactive_tab_hover();
                        let bg = bg_color
                            .unwrap_or_else(|| inactive_tab.bg_color.into())
                            .to_linear();
                        elem = elem.colors(ElementColors {
                            border: BorderColor::new(bg),
                            bg: bg.into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab.fg_color.into())
                                .to_linear()
                                .into(),
                        })
                        .hover_colors(Some(ElementColors {
                            border: BorderColor::new(
                                bg_color
                                    .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                    .to_linear(),
                            ),
                            bg: bg_color
                                .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                .to_linear()
                                .into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab_hover.fg_color.into())
                                .to_linear()
                                .into(),
                        }));
                    }

                    // Add close button (hidden by default, visible on hover)
                    let tab_bg_linear = if active {
                        bg_color
                            .unwrap_or_else(|| active_tab_colors.bg_color.into())
                            .to_linear()
                    } else {
                        bg_color
                            .unwrap_or_else(|| colors.inactive_tab().bg_color.into())
                            .to_linear()
                    };

                    // Add extra info lines (git branch, command)
                    let mut extra_lines = vec![];
                    
                    log::info!("Tab {} checking: show_extra={}, tab_idx={}, tabs_info.len()={}", 
                        tab_idx, show_extra, tab_idx, tabs_info.len());
                    
                    if show_extra && tab_idx < tabs_info.len() {
                        let tab_info = &tabs_info[tab_idx];
                        log::info!("Tab {} has active_pane: {}", tab_idx, tab_info.active_pane.is_some());
                        
                        if let Some(pane) = &tab_info.active_pane {
                            let mux = mux::Mux::get();
                            if let Some(pane_obj) = mux.get_pane(pane.pane_id) {
                                let cache_duration = self.config.tab_bar_extra_info_cache_ms;

                                // Use foreground process cwd directly — pane.title may have been
                                // modified by the running program (e.g. claude, codex) via OSC sequences.
                                let pane_cwd = pane_obj
                                    .get_foreground_process_info(mux::pane::CachePolicy::FetchImmediate)
                                    .and_then(|info| info.cwd.into_os_string().into_string().ok());
                                let (git, cmd) = get_extra_info(pane_obj.as_ref(), pane_cwd.as_deref(), cache_duration);
                                
                                log::info!("Tab {} extra info: git={:?}, cmd={:?}", tab_idx, git, cmd);
                                
                                if let Some(git) = git {
                                    let git_line = parse_status_text(&git, CellAttributes::default());
                                    let git_elem = Element::with_line(&font, &git_line, palette)
                                        .display(DisplayType::Block)
                                        .colors(ElementColors {
                                            border: BorderColor::default(),
                                            bg: tab_bg_linear.into(),
                                            text: palette.colors.0[AnsiColor::Lime as usize].to_linear().into(),
                                        });
                                    extra_lines.push(git_elem);
                                }
                                
                                if let Some(cmd) = cmd {
                                    let cmd_line = parse_status_text(&cmd, CellAttributes::default());
                                    let cmd_elem = Element::with_line(&font, &cmd_line, palette)
                                        .display(DisplayType::Block)
                                        .colors(ElementColors {
                                            border: BorderColor::default(),
                                            bg: tab_bg_linear.into(),
                                            text: palette.colors.0[AnsiColor::Grey as usize].to_linear().into(),
                                        });
                                    extra_lines.push(cmd_elem);
                                }
                                
                                log::info!("Tab {} extra_lines count: {}", tab_idx, extra_lines.len());
                            } else {
                                log::warn!("Tab {} failed to get pane object", tab_idx);
                            }
                        }
                    }

                    elem.content = match elem.content {
                        ElementContent::Children(mut kids) => {
                            let kids_before = kids.len();
                            let extra_count = extra_lines.len();
                            // Add extra info lines after title
                            kids.extend(extra_lines);
                            let kids_after = kids.len();
                            
                            log::info!("Tab {} kids: before={}, after={}, extra_lines={}", 
                                tab_idx, kids_before, kids_after, extra_count);
                            
                            // Add close button
                            if self.config.show_close_tab_button_in_tabs {
                                kids.push(make_vertical_x_button(
                                    &font, &metrics, &colors, tab_idx, active,
                                    tab_bg_linear,
                                ));
                            }
                            ElementContent::Children(kids)
                        }
                        other => {
                            log::warn!("Tab {} content is not Children: {:?}", tab_idx, std::mem::discriminant(&other));
                            other
                        }
                    };

                    tab_children.push(elem);
                }
                TabBarItem::NewTabButton => {
                    // Wrap the + icon in a full-width block so it centers
                    let icon = Element::new(
                        &font,
                        ElementContent::Poly {
                            line_width: metrics.underline_height.max(2),
                            poly: SizedPoly {
                                poly: PLUS_BUTTON,
                                width: Dimension::Pixels(
                                    metrics.cell_size.height as f32 / 2.,
                                ),
                                height: Dimension::Pixels(
                                    metrics.cell_size.height as f32 / 2.,
                                ),
                            },
                        },
                    )
                    .vertical_align(VerticalAlign::Middle);

                    let elem = Element::new(
                        &font,
                        ElementContent::Children(vec![icon]),
                    )
                    .display(DisplayType::Block)
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .margin(BoxDimension {
                        left: Dimension::Pixels(0.),
                        right: Dimension::Pixels(0.),
                        top: Dimension::Pixels(4.),
                        bottom: Dimension::Pixels(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.5),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.25),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(1.)))
                    .colors(ElementColors {
                        border: BorderColor::new(
                            new_tab_colors.bg_color.to_linear(),
                        ),
                        bg: new_tab_colors.bg_color.to_linear().into(),
                        text: new_tab_colors.fg_color.to_linear().into(),
                    })
                    .hover_colors(Some(ElementColors {
                        border: BorderColor::new(
                            new_tab_hover_colors.bg_color.to_linear(),
                        ),
                        bg: new_tab_hover_colors.bg_color.to_linear().into(),
                        text: new_tab_hover_colors.fg_color.to_linear().into(),
                    }));
                    tab_children.push(elem);
                }
                // Skip status items and window buttons in vertical mode
                _ => {}
            }
        }

        let content = ElementContent::Children(tab_children);

        // Separator color from theme (configurable via colors.tab_bar.inactive_tab_edge)
        let separator_color = colors.inactive_tab_edge().to_linear();
        let bg_linear = self.titlebar_bg_linear();

        let on_right = self.config.tab_bar_vertical_position
            == config::VerticalTabBarPosition::Right;

        // Work around box_model bug (box_model.rs:1236): right border rendering
        // uses border.left width, so both sides must be 1px. The separator side
        // gets the visible color; the other side matches the background.
        let border_colors = if on_right {
            BorderColor {
                left: separator_color,
                top: bg_linear,
                right: bg_linear,
                bottom: bg_linear,
            }
        } else {
            BorderColor {
                left: bg_linear,
                top: bg_linear,
                right: separator_color,
                bottom: bg_linear,
            }
        };

        let tabs = Element::new(&font, content)
            .display(DisplayType::Block)
            .item_type(UIItemType::TabBar(TabBarItem::None))
            .min_width(Some(Dimension::Pixels(tab_bar_width)))
            .max_width(Some(Dimension::Pixels(tab_bar_width)))
            .min_height(Some(Dimension::Pixels(
                self.dimensions.pixel_height as f32,
            )))
            .colors(ElementColors {
                border: border_colors,
                bg: bar_colors.bg,
                text: bar_colors.text,
            })
            .border(BoxDimension {
                left: Dimension::Pixels(1.),
                right: Dimension::Pixels(1.),
                top: Dimension::Pixels(0.),
                bottom: Dimension::Pixels(0.),
            })
            .padding(BoxDimension {
                left: Dimension::Pixels(3.),
                right: Dimension::Pixels(3.),
                top: Dimension::Pixels(4.),
                bottom: Dimension::Pixels(4.),
            });

        let border = self.get_os_border();

        // Always compute layout at x=0 (left edge), then translate if Right.
        // This ensures internal layout is identical to the working Left case.
        let mut computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: tab_bar_width,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(
                    border.left.get() as f32,
                    border.top.get() as f32,
                    tab_bar_width,
                    self.dimensions.pixel_height as f32
                        - (border.top + border.bottom).get() as f32,
                ),
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 10,
            },
            &tabs,
        )?;

        // Translate to right side if needed
        if on_right {
            let right_x = self.dimensions.pixel_width as f32
                - tab_bar_width
                - border.right.get() as f32
                - border.left.get() as f32;
            computed.translate(euclid::vec2(right_x, 0.));
        }

        Ok(computed)
    }

    pub fn paint_fancy_tab_bar(&self) -> anyhow::Result<Vec<UIItem>> {
        let computed = self.fancy_tab_bar.as_ref().ok_or_else(|| {
            anyhow::anyhow!("paint_fancy_tab_bar called but fancy_tab_bar is None")
        })?;
        let ui_items = computed.ui_items();

        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)?;

        Ok(ui_items)
    }
}

fn make_x_button(
    font: &Rc<LoadedFont>,
    metrics: &RenderMetrics,
    colors: &TabBarColors,
    tab_idx: usize,
    active: bool,
) -> Element {
    Element::new(
        &font,
        ElementContent::Poly {
            line_width: metrics.underline_height.max(2),
            poly: SizedPoly {
                poly: X_BUTTON,
                width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
            },
        },
    )
    // Ensure that we draw our background over the
    // top of the rest of the tab contents
    .zindex(1)
    .vertical_align(VerticalAlign::Middle)
    .float(Float::Right)
    .item_type(UIItemType::CloseTab(tab_idx))
    .hover_colors({
        let inactive_tab_hover = colors.inactive_tab_hover();
        let active_tab = colors.active_tab();

        Some(ElementColors {
            border: BorderColor::default(),
            bg: (if active {
                inactive_tab_hover.bg_color
            } else {
                active_tab.bg_color
            })
            .to_linear()
            .into(),
            text: (if active {
                inactive_tab_hover.fg_color
            } else {
                active_tab.fg_color
            })
            .to_linear()
            .into(),
        })
    })
    .padding(BoxDimension {
        left: Dimension::Cells(0.25),
        right: Dimension::Cells(0.25),
        top: Dimension::Cells(0.25),
        bottom: Dimension::Cells(0.25),
    })
    .margin(BoxDimension {
        left: Dimension::Cells(0.5),
        right: Dimension::Cells(0.),
        top: Dimension::Cells(0.),
        bottom: Dimension::Cells(0.),
    })
}

/// Close button for vertical tabs: hidden by default, visible on hover.
fn make_vertical_x_button(
    font: &Rc<LoadedFont>,
    metrics: &RenderMetrics,
    colors: &TabBarColors,
    tab_idx: usize,
    active: bool,
    tab_bg: LinearRgba,
) -> Element {
    Element::new(
        &font,
        ElementContent::Poly {
            line_width: metrics.underline_height.max(2),
            poly: SizedPoly {
                poly: X_BUTTON,
                width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
            },
        },
    )
    .zindex(1)
    .vertical_align(VerticalAlign::Top)
    .float(Float::Right)
    .item_type(UIItemType::CloseTab(tab_idx))
    // Default: invisible (text and bg match the tab background)
    .colors(ElementColors {
        border: BorderColor::default(),
        bg: tab_bg.into(),
        text: tab_bg.into(),
    })
    // Hover: reveal the X button
    .hover_colors({
        let inactive_tab_hover = colors.inactive_tab_hover();
        let active_tab = colors.active_tab();

        Some(ElementColors {
            border: BorderColor::default(),
            bg: (if active {
                inactive_tab_hover.bg_color
            } else {
                active_tab.bg_color
            })
            .to_linear()
            .into(),
            text: (if active {
                inactive_tab_hover.fg_color
            } else {
                active_tab.fg_color
            })
            .to_linear()
            .into(),
        })
    })
    .padding(BoxDimension {
        left: Dimension::Cells(0.25),
        right: Dimension::Cells(0.25),
        top: Dimension::Cells(0.25),
        bottom: Dimension::Cells(0.25),
    })
    .margin(BoxDimension {
        left: Dimension::Cells(0.5),
        right: Dimension::Cells(0.),
        top: Dimension::Cells(0.),
        bottom: Dimension::Cells(0.),
    })
}
