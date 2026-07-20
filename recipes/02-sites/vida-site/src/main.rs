fn main() -> sim_kernel::Result<()> {
    println!("{}", auto_recipe_support::assert_site("vida")?);
    Ok(())
}
