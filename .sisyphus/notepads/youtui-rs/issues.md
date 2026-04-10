### API Instance Issues
- **Problem**: Invidious instances often return HTML instead of JSON or 403 Forbidden, breaking search and import flows.
- **Solution**: 
    - Added  to detect these cases in .
    - Updated  to mark instances returning HTML as unhealthy.
    - Implemented automatic instance rotation in  for search, import, and refresh tasks.
    - Added a  channel to update the app's current instance when rotation occurs.
    - Updated  list with verified working instances (, ).
### API Instance Issues
- **Problem**: Invidious instances often return HTML instead of JSON or 403 Forbidden, breaking search and import flows.
- **Solution**: 
    - Added InvidiousError::BadInstance to detect these cases in InvidiousClient::get.
    - Updated check_invidious to mark instances returning HTML as unhealthy.
    - Implemented automatic instance rotation in src/ui/app.rs for search, import, and refresh tasks.
    - Added a settings_tx/rx channel to update the app's current instance when rotation occurs.
    - Updated INVIDIOUS_INSTANCES list with verified working instances (iv.melmac.space, invidious.materialio.us).
