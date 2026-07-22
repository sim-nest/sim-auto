fn main() -> sim_kernel::Result<()> {
    println!("{}", auto_recipe_support::assert_lane("flash")?);
    Ok(())
}
