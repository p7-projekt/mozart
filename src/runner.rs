pub trait LanguageHandler {
    /// Creates a new `LanguageHandler`.
    fn new(temp_dir: PathBuf) -> Self;

    /// Gets a reference to the temporary working directory of the current `LanguageHandler`.
    fn dir(&self) -> &PathBuf;

    /// Gets the path to the test file, the path should contain the file extension.
    fn test_file_path(&self) -> PathBuf;

    /// Gets the basic test runner before generated test cases, the solution, and the output file path are inserted.
    ///
    /// The test cases are inserted in place of the value in [`TEST_CASES_TARGET`].
    ///
    /// The output file path is inserted in place of the value in [`OUTPUT_FILE_PATH_TARGET`].
    ///
    /// The solution is inserted in place of the value in [`SOLUTION_TARGET`].
    fn base_test_code(&self) -> &str;

    /// Generates the language specific test cases.
    fn generate_test_cases(&self, test_cases: &[TestCase]) -> String;

    /// Formats a parameter to the necessary language specific syntax.
    fn format_parameter(&self, parameter: &Parameter) -> String;

    /// Runs the submission against the test cases.
    ///
    /// If the programming language is compiled, then this step **also** includes compilation of the source code.
    fn run(&self) -> Result<(), CheckError>;
}
