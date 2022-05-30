_  = function(p) return p; end;
name = _('Tcd Virtual Display');
Description = 'Tcd Virtual Display'
Viewports =
{
     Center =
     {
          x = 0;
          y = 0;
          width = 2560;
          height = 1440;
          viewDx = 0;
          viewDy = 0;
          aspect = 1.7777777777778;
     }
}

LEFT_MFCD =
{
     x = 0;
     y = 1440;
     width = 640;
     height = 640;
}

RIGHT_MFCD =
{
     x = 640;
     y = 1440;
     width = 640;
     height = 640;
}

UIMainView = Viewports.Center
GU_MAIN_VIEWPORT = Viewports.Center