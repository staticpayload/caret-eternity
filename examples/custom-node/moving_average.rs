// Custom Node Example: Moving Average
//
// This example demonstrates how to create a custom node for Caret.
// The MovingAverage node calculates the average of the last N values.

use std::collections::VecDeque;

/// Custom node that calculates moving average
pub struct MovingAverage {
    window: VecDeque<f64>,
    window_size: usize,
    sum: f64,
}

impl MovingAverage {
    /// Create a new MovingAverage node
    pub fn new(window_size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(window_size),
            window_size,
            sum: 0.0,
        }
    }

    /// Process a single value
    pub fn process(&mut self, value: f64) -> Option<f64> {
        // Add new value
        self.window.push_back(value);
        self.sum += value;

        // Remove oldest if window is full
        if self.window.len() > self.window_size {
            if let Some(oldest) = self.window.pop_front() {
                self.sum -= oldest;
            }
        }

        // Return average if we have enough data
        if self.window.len() == self.window_size {
            Some(self.sum / self.window.len() as f64)
        } else {
            None
        }
    }

    /// Get current window size
    pub fn len(&self) -> usize {
        self.window.len()
    }

    /// Clear the window
    pub fn clear(&mut self) {
        self.window.clear();
        self.sum = 0.0;
    }
}

fn main() {
    println!("Moving Average Node Example");
    println!("===========================\n");

    let mut ma = MovingAverage::new(3);
    let values = [10.0, 20.0, 30.0, 40.0, 50.0];

    for value in values {
        print!("Input: {:.1} -> ", value);
        if let Some(avg) = ma.process(value) {
            print!("Moving Average: {:.1}\n", avg);
        } else {
            print!("Collecting data... ({}/{})\n", ma.len(), ma.window_size);
        }
    }

    println!("\nTo use this node in Caret:");
    println!("1. Implement the Node trait");
    println!("2. Register the node type");
    println!("3. Use in graph.json");
}
