use egui::Context;
use lle::{
    num_complex::Complex64,
    num_traits::{Float, FromPrimitive},
};
use std::{fmt::Debug, ops::RangeInclusive};

mod history;
pub use history::History;

mod process;
pub use process::Process;

pub mod chart;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "plotters")]
pub mod plotters;

use self::chart::LleChart;

pub(crate) fn default_r_chart(index: usize) -> Option<LleChart> {
    Some(LleChart {
        name: format! {"real domain {index}"},
        kind: PlotKind::Line,
        proc: Default::default(),
        smart_bound: Some(Default::default()),
        show_history: None,
        additional: None,
    })
}

pub(crate) fn default_f_chart(index: usize) -> Option<LleChart> {
    Some(LleChart {
        name: format! {"freq domain {index}"},
        kind: PlotKind::Line,
        proc: Process::new_freq_domain(),
        smart_bound: Some(Default::default()),
        show_history: None,
        additional: None,
    })
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ViewField {
    #[serde(default)]
    pub(crate) r_chart: Option<LleChart>,
    #[serde(default)]
    pub(crate) f_chart: Option<LleChart>,
    #[serde(skip)]
    pub(crate) history: Option<History>,
    index: usize,
}

impl ViewField {
    pub(crate) fn new(index: usize) -> Self {
        Self {
            r_chart: default_r_chart(index),
            f_chart: None,
            history: None,
            index,
        }
    }
}

impl ViewField {
    pub(crate) fn toggle_record_his(&mut self, ui: &mut egui::Ui, data: &[Complex64]) {
        let index = self.index;
        if crate::toggle_option_with(
            ui,
            &mut self.history,
            format!("Record history {index}"),
            || Some(History::new(data.to_vec())),
        )
        .clicked()
            && self.history.is_none()
        {
            for c in [self.r_chart.as_mut(), self.f_chart.as_mut()]
                .into_iter()
                .flatten()
            {
                c.show_history = None;
            }
        }
    }

    pub(crate) fn log_his(&mut self, data: &[Complex64]) {
        if let Some(ref mut s) = self.history {
            s.push(data)
        }
    }

    pub(crate) fn show_which(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            crate::toggle_option_with(ui, &mut self.r_chart, "real domain", || {
                default_r_chart(self.index)
            });
            crate::toggle_option_with(ui, &mut self.f_chart, "freq domain", || {
                default_f_chart(self.index)
            });
        });
    }
    pub(crate) fn visualize_state(
        &mut self,
        data: &[Complex64],
        ctx: &Context,
        running: bool,
        #[cfg(feature = "gpu")] render_state: &eframe::egui_wgpu::RenderState,
    ) {
        puffin::profile_function!();
        LleChart::plot_on_new_window(
            &mut self.r_chart,
            data,
            ctx,
            running,
            &self.history,
            #[cfg(feature = "gpu")]
            render_state,
        );
        LleChart::plot_on_new_window(
            &mut self.f_chart,
            data,
            ctx,
            running,
            &self.history,
            #[cfg(feature = "gpu")]
            render_state,
        );
    }
}

#[derive(
    Debug,
    Clone,
    enum_iterator::Sequence,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum PlotKind {
    Line,
    Points,
}

impl PlotKind {
    pub fn desc(&self) -> &str {
        match self {
            PlotKind::Line => "Line",
            PlotKind::Points => "Points",
        }
    }
    pub fn controller(&mut self, ui: &mut egui::Ui) {
        enum_iterator::all::<PlotKind>().for_each(|s| {
            if ui.selectable_label(self == &s, s.desc()).clicked() {
                *self = s;
            }
        })
    }

    pub(crate) fn plot<'a>(
        &self,
        plot: egui_plot::Plot<'_>,
        ui: &mut egui::Ui,
        bound: Option<egui_plot::PlotBounds>,
        elements: impl Iterator<Item = (egui_plot::PlotPoints, Option<&'a str>)>,
    ) -> egui_plot::PlotResponse<()> {
        plot.show(ui, |plot_ui| {
            if let Some(bound) = bound {
                plot_ui.set_plot_bounds(bound);
            }

            match self {
                PlotKind::Line => {
                    elements.for_each(|(e, d)| {
                        if let Some(d) = d {
                            plot_ui.line(egui_plot::Line::new(e).name(d));
                        } else {
                            plot_ui.line(egui_plot::Line::new(e));
                        }
                    });
                }
                PlotKind::Points => {
                    elements.for_each(|(e, d)| {
                        if let Some(d) = d {
                            plot_ui.points(egui_plot::Points::new(e).name(d));
                        } else {
                            plot_ui.points(egui_plot::Points::new(e));
                        }
                    });
                }
            }
        })
    }
}
