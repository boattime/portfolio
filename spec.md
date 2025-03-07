# Portfolio Project Specification: Rust-Based Observability and SRE Showcase

## Project Overview
This specification outlines a portfolio website showcasing advanced Rust programming and observability/SRE concepts. The site will be entirely static HTML/CSS with pre-rendered metrics and visualizations, regenerated every 30 seconds. The UI will feature a terminal-like interface using pure CSS, with no JavaScript dependencies.

## Core Requirements

### Functional Requirements
1. Generate a completely static website using Rust
2. Display actual performance metrics, traces, and logs
3. Feature a terminal-like interface using pure CSS only
4. Regenerate the static site every 30 seconds
5. Host on AWS S3 with CloudFront distribution
6. Support macOS and Linux operating systems
7. Demonstrate concurrent processing for site generation
8. Visualize how concurrent components operate

### Non-Functional Requirements
1. Site must achieve 99.999% or higher availability
2. Zero JavaScript usage (pure HTML/CSS only)
3. Extremely fast page load times
4. Optimized asset sizes

## Technical Architecture

### System Components

1. **Metric Collector**
   - Pulls metrics from Prometheus at regular intervals
   - Stores metrics in efficient Rust data structures
   - Handles connection failures gracefully with retry logic

2. **Trace Aggregator**
   - Collects trace spans from application components
   - Organizes trace data for visualization
   - Creates latency breakdowns and dependency maps

3. **Log Parser**
   - Processes application logs with a focus on site generation events
   - Extracts timing information, errors, and key events
   - Formats logs for terminal-style display

4. **Concurrency Manager**
   - Implements thread pools and work distribution
   - Uses Rust channels for inter-thread communication
   - Coordinates parallel generation of different site sections
   - Demonstrates Rust's ownership model and concurrency patterns

5. **Template Engine**
   - Creates terminal-like interface using HTML/CSS templates
   - Applies pure CSS styling for terminal aesthetics
   - Handles template rendering with efficient algorithms

6. **Static Asset Optimizer**
   - Compresses and optimizes images and other assets
   - Minimizes CSS files
   - Ensures optimized file sizes for fast loading

7. **AWS Deployment Engine**
   - Uploads generated content to S3
   - Invalidates CloudFront caches as needed
   - Handles AWS API interactions efficiently

8. **Visualization Generator**
   - Creates SVG/CSS-based visualizations for metrics
   - Generates visual representations of traces and dependencies
   - Implements terminal-style graphs and charts without JavaScript

9. **Scheduler**
   - Triggers site regeneration every 30 seconds
   - Maintains timing accuracy
   - Handles scheduling conflicts

10. **Concurrency Visualizer**
    - Creates visual representations of concurrent processes
    - Shows thread activity and work distribution
    - Demonstrates performance gains from concurrency

### Data Flow

1. Scheduler triggers regeneration cycle every 30 seconds
2. Concurrency Manager distributes work across multiple threads
3. Metric Collector pulls latest data from Prometheus
4. Trace Aggregator processes latest traces
5. Log Parser extracts recent log data
6. Template Engine renders HTML/CSS based on collected data
7. Static Asset Optimizer processes all assets
8. Visualization Generator creates SVG-based visualizations
9. AWS Deployment Engine uploads to S3 and refreshes CloudFront
10. Concurrency Visualizer records and displays thread activity

## Implementation Details

### Observability Integration
- Prometheus integration for metrics collection
- Display of key metrics:
  - Requests over time
  - Asset sizes
  - Last update timestamps
  - Site generation speed
  - Site availability statistics
- Visualization of actual trace spans
- Dependency maps showing component relationships
- Latency breakdowns across system components

### Rust Techniques Implementation
- **Concurrency patterns**:
  - Thread pools for parallel processing
  - Channels for thread communication
  - Mutex and Arc for shared state management
  - Async/await patterns where appropriate
- Ownership model demonstration through safe memory management
- Error handling with Result and Option types
- High-performance parsing and processing

### Terminal-Like Interface
- Pure CSS styling without JavaScript
- Terminal-inspired color scheme (dark background, light text)
- Monospace fonts for authentic terminal feel
- Command-line style output formatting
- Static display of metrics resembling terminal output
- Pseudo-command prompt styling for section headers

### AWS Deployment
- S3 bucket for static content hosting
- CloudFront distribution for global delivery
- Automated deployment process

## Development Guidelines

### Code Structure
- Modular architecture with clearly separated components
- Rust crate organization following industry best practices
- Clean API boundaries between modules
- Comprehensive logging and error handling

### Error Handling Strategy
- Graceful degradation if metrics collection fails
- Retry mechanisms for transient errors
- Fallback to cached data when new data isn't available
- Comprehensive error logging

### Testing Plan
1. **Unit Testing**:
   - Test each component in isolation
   - Mock dependencies for isolated component testing
   - Test error conditions and edge cases

2. **Integration Testing**:
   - Test interaction between components
   - Verify data flow through the system
   - Test concurrency behavior under load

3. **End-to-End Testing**:
   - Verify complete site generation process
   - Test deployment to AWS
   - Validate metrics displayed match source data

4. **Performance Testing**:
   - Measure site generation times
   - Test under various load conditions
   - Verify concurrency benefits

## Deliverables
1. Rust application source code with comprehensive documentation
2. Build and deployment scripts
3. AWS infrastructure configuration (can be IaC with Terraform or similar)
4. Setup instructions for local development environment
5. Testing suite and documentation

## Future Enhancements (Not in Initial Scope)
1. SRE self-healing demonstration features
2. Performance benchmarks comparing with other languages
3. Extended monitoring of the generation process itself
4. Notification systems for metric anomalies
