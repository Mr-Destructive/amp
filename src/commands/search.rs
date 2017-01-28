use errors::*;
use commands::{self, Result};
use models::application::{Application, Mode};

pub fn move_to_previous_result(app: &mut Application) -> Result {
    let mut moved = false;

    {
        let query = app
            .search_query
            .as_ref()
            .ok_or("Can't navigate results without a search query")?;
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        let positions = buffer.search(query);
        for position in positions.iter().rev() {
            if position < &*buffer.cursor {
                buffer.cursor.move_to(*position);

                // We've found one; track that and stop looking.
                moved = true;
                break;
            }
        }

        if !moved {
            // There's nothing before the cursor, so wrap
            // to the last match, if there are any at all.
            if let Some(position) = positions.last() {
                buffer.cursor.move_to(*position);
                moved = true;
            }
        }
    }

    if moved {
        commands::view::scroll_cursor_to_center(app);
    }

    Ok(())
}

pub fn move_to_next_result(app: &mut Application) -> Result {
    let mut moved = false;

    {
        let query = app
            .search_query
            .as_ref()
            .ok_or("Can't navigate results without a search query")?;
        let buffer = app.workspace.current_buffer().ok_or(BUFFER_MISSING)?;
        let positions = buffer.search(query);

        // Try to find a result after the cursor.
        for position in &positions {
            if position > &buffer.cursor {
                buffer.cursor.move_to(*position);

                // We've found one; track that and stop looking.
                moved = true;
                break;
            }
        }

        if !moved {
            // We haven't found anything after the cursor, so wrap
            // to the first match, if there are any matches at all.
            if let Some(position) = positions.first() {
                buffer.cursor.move_to(*position);
                moved = true;
            }
        }
    }

    if moved {
        commands::view::scroll_cursor_to_center(app);
    }

    Ok(())
}

pub fn accept_query(app: &mut Application) -> Result {
    let query = match app.mode {
        Mode::SearchInsert(ref mode) => Some(mode.input.clone()),
        _ => None,
    }.ok_or("Can't accept search query outside of search mode")?;

    commands::application::switch_to_normal_mode(app);
    app.search_query = Some(query);
    move_to_next_result(app);

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate scribe;

    use scribe::Buffer;
    use scribe::buffer::Position;
    use models::Application;
    use models::application::Mode;
    use commands;

    #[test]
    fn move_to_previous_result_moves_cursor_to_previous_result() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Move beyond the second result.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 2,
            offset: 0,
        });

        // Reverse to the second result.
        commands::search::move_to_previous_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 1,
                       offset: 0,
                   });
    }

    #[test]
    fn move_to_previous_result_wraps_to_the_end_of_the_document() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Reverse to the previous result, forcing the wrap.
        commands::search::move_to_previous_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 2,
                       offset: 0,
                   });
    }

    #[test]
    fn move_to_next_result_moves_cursor_to_next_result() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Advance to the second result.
        commands::search::move_to_next_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 4,
                   });
    }

    #[test]
    fn move_to_next_result_wraps_to_the_start_of_the_document() {
        // Build a workspace with a buffer and text.
        let mut app = Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Set the search query for the application.
        app.search_query = Some("ed".to_string());

        // Move to the end of the document.
        app.workspace.current_buffer().unwrap().cursor.move_to(Position {
            line: 2,
            offset: 0,
        });

        // Advance to the next result, forcing the wrap.
        commands::search::move_to_next_result(&mut app);

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 4,
                   });
    }

    #[test]
    fn accept_query_sets_application_search_query_switches_to_normal_mode_and_moves_to_first_match
        () {
        let mut app = ::models::Application::new().unwrap();
        let mut buffer = Buffer::new();
        buffer.insert("amp editor\nedit\nedit");
        app.workspace.add_buffer(buffer);

        // Enter search mode and add a search value.
        commands::application::switch_to_search_insert_mode(&mut app);
        match app.mode {
            Mode::SearchInsert(ref mut mode) => mode.input = "ed".to_string(),
            _ => (),
        };
        commands::search::accept_query(&mut app);

        // Ensure that we're in normal mode.
        assert!(match app.mode {
            ::models::application::Mode::Normal => true,
            _ => false,
        });

        // Ensure that the search query is properly set.
        assert_eq!(app.search_query, Some("ed".to_string()));

        // Ensure the buffer cursor is at the expected position.
        assert_eq!(*app.workspace.current_buffer().unwrap().cursor,
                   Position {
                       line: 0,
                       offset: 4,
                   });
    }
}
