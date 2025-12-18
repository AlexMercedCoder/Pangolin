#!/usr/bin/env python3
"""
Script to combine all markdown files from a subdirectory into a single file AND a PDF.

Usage:
    python combine_markdown_pdf.py <subdirectory> <output_filename>
    
Example:
    python combine_markdown_pdf.py docs combined_docs
    
This will create:
- combined_docs.md
- combined_docs.pdf
"""

import argparse
import os
from pathlib import Path
import sys
from fpdf import FPDF
from markdown_it import MarkdownIt

def find_markdown_files(directory: str) -> list[Path]:
    """
    Recursively find all markdown files in the given directory.
    """
    directory_path = Path(directory)
    
    if not directory_path.exists():
        raise FileNotFoundError(f"Directory '{directory}' does not exist")
    
    if not directory_path.is_dir():
        raise NotADirectoryError(f"'{directory}' is not a directory")
    
    # Find all .md files recursively
    markdown_files = sorted(directory_path.rglob("*.md"))
    
    return markdown_files


def combine_markdown_files(markdown_files: list[Path], output_file: str) -> str:
    """
    Combine all markdown files into a single output file.
    Returns the combined content.
    """
    # Ensure output filename has .md extension
    if not output_file.endswith('.md'):
        output_file = f"{output_file}.md"
    
    output_path = Path(output_file)
    combined_content = ""
    
    with open(output_path, 'w', encoding='utf-8') as outfile:
        for i, md_file in enumerate(markdown_files):
            # Add separator between files
            if i > 0:
                sep = "\n\n---\n\n"
                outfile.write(sep)
                combined_content += sep
            
            # Write a header indicating the source file
            header = f"<!-- Source: {md_file} -->\n\n"
            outfile.write(header)
            # We don't necessarily want the comment in the PDF content, but maybe useful?
            # Let's skip the comment in the content passed to PDF to keep it clean, 
            # or maybe add a visual header?
            # Let's add a visual header for the file in the PDF content
            file_header = f"# Source: {md_file.name}\n\n"
            combined_content += file_header
            
            # Read and write the content of the markdown file
            try:
                with open(md_file, 'r', encoding='utf-8') as infile:
                    content = infile.read()
                    outfile.write(content)
                    combined_content += content
            except Exception as e:
                print(f"Warning: Could not read {md_file}: {e}", file=sys.stderr)
                continue
    
    print(f"Successfully combined {len(markdown_files)} markdown files into {output_path}")
    return combined_content


class PDFRenderer(FPDF):
    def __init__(self):
        super().__init__()
        self.set_auto_page_break(auto=True, margin=15)
        
        # Add Unicode font
        font_dir = "/usr/share/fonts/truetype/dejavu"
        font_path = f"{font_dir}/DejaVuSans.ttf"
        
        if os.path.exists(font_path):
            self.add_font("DejaVu", fname=font_path)
            
            # Add variants if they exist
            if os.path.exists(f"{font_dir}/DejaVuSans-Bold.ttf"):
                self.add_font("DejaVu", style="B", fname=f"{font_dir}/DejaVuSans-Bold.ttf")
            if os.path.exists(f"{font_dir}/DejaVuSans-Oblique.ttf"):
                self.add_font("DejaVu", style="I", fname=f"{font_dir}/DejaVuSans-Oblique.ttf")
            if os.path.exists(f"{font_dir}/DejaVuSans-BoldOblique.ttf"):
                self.add_font("DejaVu", style="BI", fname=f"{font_dir}/DejaVuSans-BoldOblique.ttf")
                
            # Add Mono font
            if os.path.exists(f"{font_dir}/DejaVuSansMono.ttf"):
                self.add_font("DejaVuMono", fname=f"{font_dir}/DejaVuSansMono.ttf")
                
            self.set_font("DejaVu", size=12)
        else:
            print(f"Warning: Unicode font not found at {font_path}. Using Helvetica (no emoji support).", file=sys.stderr)
            self.set_font("helvetica", size=12)
            
        self.add_page()
        
    def header(self):
        # Optional header
        pass
        
    def footer(self):
        self.set_y(-15)
        # Use current font family
        self.set_font(size=8)
        self.cell(0, 10, f"Page {self.page_no()}", align="C")

    def render_markdown(self, md_content):
        if self.page_no() == 0:
            self.add_page()
            
        md = MarkdownIt()
        tokens = md.parse(md_content)
        
        # Determine current font family
        font_family = "DejaVu" if "dejavu" in self.fonts else "helvetica"
        mono_font = "DejaVuMono" if "dejavumono" in self.fonts else "courier"
        
        for i, token in enumerate(tokens):
            if token.type == "heading_open":
                level = int(token.tag[1])
                size = 24 - (level * 2) # h1=22, h2=20, ...
                self.set_font(font_family, "B" if font_family == "helvetica" else "", size) 
                self.ln(5)
                
            elif token.type == "heading_close":
                self.set_font(font_family, size=12)
                self.ln(5)
                
            elif token.type == "paragraph_open":
                self.ln(2)
                
            elif token.type == "paragraph_close":
                self.ln(2)
                
            elif token.type == "inline":
                content = token.content
                try:
                    self.multi_cell(0, 5, content, markdown=True)
                except Exception:
                    self.multi_cell(0, 5, content)
                    
            elif token.type == "fence": # Code block
                self.set_font(mono_font, size=10)
                self.set_fill_color(240, 240, 240)
                self.multi_cell(0, 5, token.content, fill=True)
                self.set_fill_color(255, 255, 255)
                self.set_font(font_family, size=12)
                self.ln(5)
                
            elif token.type == "bullet_list_open":
                pass 
            
            elif token.type == "list_item_open":
                self.write(5, "- ") 


def generate_pdf(content: str, output_filename: str):
    """Generate PDF from markdown content."""
    if not output_filename.endswith('.pdf'):
        output_filename = f"{output_filename}.pdf"
        
    pdf = PDFRenderer()
    try:
        pdf.render_markdown(content)
        pdf.output(output_filename)
        print(f"Successfully generated PDF: {output_filename}")
    except Exception as e:
        print(f"Error generating PDF: {e}", file=sys.stderr)


def main():
    """Main function to parse arguments and execute the script."""
    parser = argparse.ArgumentParser(
        description="Combine all markdown files from a subdirectory into a single file and PDF.",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        'subdirectory',
        help='Path to the subdirectory containing markdown files'
    )
    
    parser.add_argument(
        'output_filename',
        help='Name for the output file (without extension)'
    )
    
    args = parser.parse_args()
    
    try:
        # Find all markdown files
        markdown_files = find_markdown_files(args.subdirectory)
        
        if not markdown_files:
            print(f"No markdown files found in '{args.subdirectory}'", file=sys.stderr)
            sys.exit(1)
        
        print(f"Found {len(markdown_files)} markdown files in '{args.subdirectory}'")
        
        # Combine them into a single file and get content
        # Remove extension if provided to ensure we add .md and .pdf correctly
        base_name = args.output_filename
        if base_name.endswith('.md'):
            base_name = base_name[:-3]
        elif base_name.endswith('.pdf'):
            base_name = base_name[:-4]
            
        combined_content = combine_markdown_files(markdown_files, base_name)
        
        # Generate PDF
        generate_pdf(combined_content, base_name)
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
