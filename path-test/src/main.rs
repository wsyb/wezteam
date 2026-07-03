fn main() {
    // 测试 1: C:\Users\10529\ 正常路径
    println!("=== 测试 1: C:\\Users\\10529\\ ===");
    let path1 = std::path::Path::new(r"C:\Users\10529\");
    let comps1: Vec<_> = path1.components().collect();
    println!("  所有组件: {:?}", comps1);
    println!("  组件数量: {}", comps1.len());

    // 模拟 simplify_path 逻辑
    let last_6: Vec<_> = comps1.iter().rev().take(6).cloned().collect();
    println!("  最后6个(反转后): {:?}", last_6);
    let mut final_comps: Vec<_> = last_6.into_iter().rev().collect();
    println!("  最终组件: {:?}", final_comps);
    let result1 = final_comps.iter()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/");
    println!("  结果: {}", result1);

    // 测试 2: 长路径
    println!("\n=== 测试 2: C:\\Users\\10529\\Documents\\Projects\\wezterm\\src\\fancy_tab_bar.rs ===");
    let path2 = std::path::Path::new(r"C:\Users\10529\Documents\Projects\wezterm\src\fancy_tab_bar.rs");
    let comps2: Vec<_> = path2.components().collect();
    println!("  所有组件: {:?}", comps2);
    println!("  组件数量: {}", comps2.len());

    let last_6_2: Vec<_> = comps2.iter().rev().take(6).cloned().collect();
    let mut final_comps2: Vec<_> = last_6_2.into_iter().rev().collect();
    let result2 = final_comps2.iter()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/");
    println!("  结果: {}", result2);

    // 测试 3: 反转路径
    println!("\n=== 测试 3: wezteam\\work\\D: (反转路径) ===");
    let path3 = std::path::Path::new(r"wezteam\work\D:");
    let comps3: Vec<_> = path3.components().collect();
    println!("  所有组件: {:?}", comps3);
    println!("  组件数量: {}", comps3.len());
}
