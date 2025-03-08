# Portfolio Project Specification: Rust-Based Observability and SRE Showcase

## Project Overview
This specification outlines a portfolio website showcasing advanced Rust programming and observability/SRE concepts. The site will be generated in both HTML/CSS and plain text formats, with pre-rendered metrics and visualizations, regenerated every 30 seconds. The UI will feature a terminal-like interface using pure CSS for HTML format and ASCII art for text format, with no JavaScript dependencies.

## Core Requirements

### Functional Requirements
1. Generate a static website using Rust in two formats:
   - HTML/CSS for browser viewing
   - Plain text for terminal/curl access
2. Display actual performance metrics, traces, and logs in both formats
3. Feature a terminal-like interface using pure CSS for HTML and ASCII art for text
4. Regenerate the static site every 30 seconds
5. Host on AWS S3 with CloudFront distribution
6. Support macOS and Linux operating systems
7. Demonstrate concurrent processing for site generation
8. Visualize how concurrent components operate
9. Ensure consistent visual design between HTML and text formats

### Non-Functional Requirements
1. Site must achieve 99.999% or higher availability
2. Zero JavaScript usage (pure HTML/CSS only)
3. Extremely fast page load times
4. Optimized asset sizes
5. Consistent user experience across both HTML and text formats

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
   - Creates format-agnostic template representation of content
   - Supports rendering to multiple output formats (HTML, text)
   - Applies format-specific styling (CSS for HTML, ASCII art for text)
   - Handles template rendering with efficient algorithms

6. **Renderer System**
   - Implements a Renderer trait with format-specific implementations
   - HtmlRenderer: Generates HTML with CSS styling
   - TextRenderer: Generates plain text with ASCII art styling
   - Ensures consistent visual appearance between formats

7. **Static Asset Optimizer**
   - Compresses and optimizes images and other assets
   - Minimizes CSS files
   - Ensures optimized file sizes for fast loading

8. **AWS Deployment Engine**
   - Uploads generated content to S3
   - Invalidates CloudFront caches as needed
   - Handles AWS API interactions efficiently

9. **Visualization Generator**
   - Creates SVG/CSS-based visualizations for HTML format
   - Generates ASCII art visualizations for text format
   - Implements terminal-style graphs and charts without JavaScript

10. **Scheduler**
    - Triggers site regeneration every 30 seconds
    - Maintains timing accuracy
    - Handles scheduling conflicts

11. **Concurrency Visualizer**
    - Creates visual representations of concurrent processes
    - Shows thread activity and work distribution
    - Demonstrates performance gains from concurrency
    - Outputs in both HTML and text formats

### Data Flow

1. Scheduler triggers regeneration cycle every 30 seconds
2. Concurrency Manager distributes work across multiple threads
3. Metric Collector pulls latest data from Prometheus
4. Trace Aggregator processes latest traces
5. Log Parser extracts recent log data
6. Template Engine processes templates into format-agnostic intermediate representation
7. Renderers convert intermediate representation to HTML and text outputs
8. Static Asset Optimizer processes all assets
9. Visualization Generator creates visualizations for both formats
10. AWS Deployment Engine uploads to S3 and refreshes CloudFront
11. Concurrency Visualizer records and displays thread activity

## Implementation Details

### Multi-Format Templating System
- Format-agnostic template model capturing content structure, not presentation
- Specialized renderers for different output formats:
  - HTML Renderer: Produces HTML with CSS styling
  - Text Renderer: Produces plain text with ASCII art elements
- Block-based intermediate representation of template content
- Consistent styling and appearance between formats
- Single source of truth for all output formats

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
- Trait-based polymorphism for renderer implementations

### Terminal-Like Interface
- HTML format:
  - Pure CSS styling without JavaScript
  - Terminal-inspired color scheme (dark background, light text)
  - Monospace fonts for authentic terminal feel
  - Command-line style output formatting
- Text format:
  - ASCII art for frames, borders, and tables
  - Unicode box drawing characters for terminal frames
  - Proportional spacing to maintain visual structure
  - Command prompts and output formatting

### AWS Deployment
- S3 bucket for static content hosting
- CloudFront distribution for global delivery
- Automated deployment process
- Content negotiation for HTML vs text format delivery

## Development Guidelines

### Code Structure
- Modular architecture with clearly separated components
- Rust crate organization following industry best practices
- Clean API boundaries between modules
- Trait-based design for renderer implementations
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
   - Test both HTML and text renderers independently

2. **Integration Testing**:
   - Test interaction between components
   - Verify data flow through the system
   - Test concurrency behavior under load
   - Verify consistency between output formats

3. **End-to-End Testing**:
   - Verify complete site generation process for both formats
   - Test deployment to AWS
   - Validate metrics displayed match source data
   - Compare HTML and text outputs for consistency

4. **Performance Testing**:
   - Measure site generation times
   - Test under various load conditions
   - Verify concurrency benefits
   - Compare performance between HTML and text generation

## Deliverables
1. Rust application source code with comprehensive documentation
2. Build and deployment scripts
3. AWS infrastructure configuration (can be IaC with Terraform or similar)
4. Setup instructions for local development environment
5. Testing suite and documentation
6. Sample curl commands to interact with the text format

## Future Enhancements (Not in Initial Scope)
1. SRE self-healing demonstration features
2. Performance benchmarks comparing with other languages
3. Extended monitoring of the generation process itself
4. Notification systems for metric anomalies
5. Additional output formats (JSON, CSV, etc.)
