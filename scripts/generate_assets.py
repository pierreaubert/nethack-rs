import json
import argparse
import os
import requests
import base64
import time
from pathlib import Path

# Common style block to ensure consistency across all icons
STYLE_MODIFIER = (
    "high-quality stylized digital illustration for a modern roguelike game, "
    "smooth shading and gradients, anti-aliased edges, vibrant professional color palette, "
    "clean centered composition, white background (to be transparent)."
)

def generate_prompt(identifier, resolution):
    """Generate a prompt for the model based on item identifiers."""
    parts = []
    
    # Extract properties
    item_class = identifier.get("class")
    obj_type = identifier.get("object_type")
    material = identifier.get("material")
    is_id = identifier.get("is_identified")
    artifact = identifier.get("artifact")
    
    # Build core description
    if artifact:
        parts.append(f"the legendary artifact {artifact}")
    elif obj_type:
        parts.append(f"a {obj_type}")
    elif item_class:
        parts.append(f"a generic {item_class}")
    else:
        parts.append("a mysterious object")
        
    if material:
        parts.append(f"made of {material}")
        
    if is_id is False:
        parts.append("unidentified appearance")
    elif is_id is True:
        parts.append("identified appearance with magical glow")

    description = ", ".join(parts)
    prompt = f"Create a {description} as a {resolution}x{resolution} game asset. {STYLE_MODIFIER}"
    return prompt

def call_api(url, headers, payload, max_retries=5):
    """Call the API with exponential backoff for rate limiting."""
    last_response = None
    for i in range(max_retries):
        try:
            # print(f"  DEBUG: API Call Attempt {i+1}/{max_retries}")
            response = requests.post(url, headers=headers, json=payload, timeout=60)
            last_response = response
            
            if response.status_code == 429:
                wait_time = (2 ** i) + 5
                print(f"  Rate limited (429). Retrying in {wait_time}s...")
                time.sleep(wait_time)
                continue
                
            return response
        except requests.exceptions.RequestException as e:
            print(f"  Network error: {e}")
            if i == max_retries - 1:
                raise e
            time.sleep(2 ** i)
            
    return last_response

def main():
    parser = argparse.ArgumentParser(description="Batch generate item icons using Google Gemini API.")
    parser.add_argument("--mapping", required=True, help="Path to mapping.json")
    parser.add_argument("--output", required=True, help="Output directory for sprites")
    parser.add_argument("--dry-run", action="store_true", help="Print prompts without executing")
    parser.add_argument("--api-key", help="Google API Key (or set GOOGLE_API_KEY env var)")
    parser.add_argument("--limit", type=int, help="Limit the number of items to generate")
    parser.add_argument("--force", action="store_true", help="Force re-generation of existing files")
    parser.add_argument("--reference-image", help="Path to a reference image to maintain style")
    parser.add_argument("--resolution", type=int, default=512, help="Output resolution (default: 512)")
    parser.add_argument("--model", default="imagen-3.0-generate-001", help="Model name (default: imagen-3.0-generate-001)")
    
    args = parser.parse_args()
    
    api_key = args.api_key or os.environ.get("GOOGLE_API_KEY")
    if not api_key and not args.dry_run:
        print("Error: Google API Key is required. Set GOOGLE_API_KEY or use --api-key.")
        return

    reference_b64 = None
    if args.reference_image:
        if not os.path.exists(args.reference_image):
            print(f"Error: Reference image {args.reference_image} not found.")
            return
        with open(args.reference_image, "rb") as f:
            reference_b64 = base64.b64encode(f.read()).decode("utf-8")

    if not os.path.exists(args.mapping):
        print(f"Error: Mapping file {args.mapping} not found.")
        return

    with open(args.mapping, 'r') as f:
        mapping_data = json.load(f)
        
    os.makedirs(args.output, exist_ok=True)
    
    mappings = mapping_data.get("mappings", [])
    
    # Filter out existing if not forced
    if not args.force:
        remaining_mappings = []
        for entry in mappings:
            sprite_path = entry.get("icon", {}).get("bevy_sprite")
            if sprite_path:
                target_path = os.path.join(args.output, os.path.basename(sprite_path))
                if not os.path.exists(target_path):
                    remaining_mappings.append(entry)
        mappings = remaining_mappings

    if args.limit:
        mappings = mappings[:args.limit]

    print(f"Processing {len(mappings)} mappings...")
    
    # Endpoint and headers
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{args.model}:generateContent"
    headers = {
        "Content-Type": "application/json",
        "X-goog-api-key": api_key
    }
    
    for entry in mappings:
        identifier = entry.get("identifier", {})
        icon_def = entry.get("icon", {})
        sprite_path = icon_def.get("bevy_sprite")
        
        if not sprite_path:
            continue
            
        filename = os.path.basename(sprite_path)
        target_path = os.path.join(args.output, filename)
        
        prompt = generate_prompt(identifier, args.resolution)
        
        print(f"Generating: {filename}")
        print(f"  Prompt: {prompt}")
        
        if args.dry_run:
            continue
            
        # Build multi-part content if reference image exists
        parts = []
        if reference_b64:
            parts.append({
                "inline_data": {
                    "mime_type": "image/png",
                    "data": reference_b64
                }
            })
            parts.append({"text": f"Generate a new item icon matching the style of the provided reference image. The new item is: {prompt}"})
        else:
            parts.append({"text": prompt})

        # Payload for image generation
        if "imagen" in args.model.lower():
            # Standard Imagen 3 payload
            payload = {
                "instances": [
                    {"prompt": prompt}
                ],
                "parameters": {
                    "sampleCount": 1,
                    "aspectRatio": "1:1",
                    "includeSafetyAttributes": False
                }
            }
        else:
            # Gemini-style payload
            payload = {
                "contents": [
                    {
                        "parts": parts
                    }
                ],
                "generationConfig": {
                    "response_modalities": ["IMAGE"]
                }
            }
        
        try:
            response = call_api(url, headers, payload)
            
            if response is None:
                print(f"  Error: No response from API (max retries reached or internal error)")
                continue
                
            if response.status_code != 200:
                print(f"  Error: {response.status_code}")
                print(f"  Response: {response.text}")
                continue
            
            result = response.json()
            
            # Extract image data from candidates
            saved = False
            candidates = result.get("candidates", [])
            if candidates:
                content_parts = candidates[0].get("content", {}).get("parts", [])
                for part in content_parts:
                    if "inlineData" in part:
                        img_data = part["inlineData"].get("data")
                        if img_data:
                            image_bytes = base64.b64decode(img_data)
                            with open(target_path, 'wb') as img_f:
                                img_f.write(image_bytes)
                            print(f"  Saved to {target_path}")
                            saved = True
                            break
            
            if not saved:
                print(f"  Warning: No image data found in response for {filename}")
                if "error" in result:
                    print(f"  API Error: {result['error'].get('message')}")
                
        except Exception as e:
            print(f"  Error generating {filename}: {e}")

if __name__ == "__main__":
    main()
